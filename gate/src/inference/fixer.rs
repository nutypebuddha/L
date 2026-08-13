use super::result::TokenFix;

pub struct TokenFixer;

impl TokenFixer {
    pub fn new() -> Self {
        TokenFixer
    }

    pub fn fix(&self, text: &str, context: &str) -> (String, Vec<TokenFix>) {
        let mut fixes = Vec::new();
        let mut result = text.to_string();

        let (fixed, math_fixes) = self.fix_math(&result, context);
        result = fixed;
        fixes.extend(math_fixes);

        let (fixed, unit_fixes) = self.fix_units(&result, context);
        result = fixed;
        fixes.extend(unit_fixes);

        let (fixed, typo_fixes) = self.fix_typos(&result);
        result = fixed;
        fixes.extend(typo_fixes);

        let (fixed, consistency_fixes) = self.fix_consistency(&result, context);
        result = fixed;
        fixes.extend(consistency_fixes);

        (result, fixes)
    }

    fn fix_math(&self, text: &str, _context: &str) -> (String, Vec<TokenFix>) {
        let mut fixes = Vec::new();
        let mut result = text.to_string();

        // Iterate until no more fixes are found (handles offset shifts)
        loop {
            let bytes = result.as_bytes();
            let mut eq_positions = Vec::new();
            for i in 0..bytes.len() {
                if bytes[i] == b'=' {
                    if i > 0 && bytes[i - 1] == b'!' {
                        continue;
                    }
                    if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                        continue;
                    }
                    eq_positions.push(i);
                }
            }

            let mut fixed_one = false;
            for eq_pos in eq_positions {
                let mut lhs_start = eq_pos;
                let mut depth = 0i32;
                for i in (0..eq_pos).rev() {
                    match bytes[i] {
                        b')' | b']' => depth += 1,
                        b'(' | b'[' => depth -= 1,
                        _ if depth == 0 => {
                            if bytes[i] == b','
                                || bytes[i] == b'.'
                                || bytes[i] == b';'
                                || bytes[i] == b':'
                                || bytes[i] == b'?'
                                || bytes[i] == b'!'
                            {
                                lhs_start = i + 1;
                                break;
                            }
                            if i == 0 {
                                lhs_start = 0;
                            }
                        }
                        _ => {}
                    }
                    if i == 0 {
                        lhs_start = 0;
                    }
                }

                let rhs_end = bytes[eq_pos + 1..]
                    .iter()
                    .position(|&b| {
                        b == b','
                            || b == b'.'
                            || b == b';'
                            || b == b':'
                            || b == b'?'
                            || b == b'!'
                            || b == b'\n'
                    })
                    .map_or(result.len(), |p| eq_pos + 1 + p);

                let lhs_str = result[lhs_start..eq_pos].trim();
                let rhs_str = result[eq_pos + 1..rhs_end].trim();

                if let Ok(rhs_val) = rhs_str.parse::<f64>() {
                    if let Some(lhs_val) = self.eval_expr(lhs_str) {
                        if (lhs_val - rhs_val).abs() > 0.001 {
                            let original_eq = format!("{} = {}", lhs_str, rhs_str);
                            let fixed_eq = format!("{} = {}", lhs_str, self.format_number(lhs_val));

                            let token_fix = TokenFix::new(
                                &original_eq,
                                &fixed_eq,
                                "Math verification failed",
                                0.95,
                            );
                            fixes.push(token_fix);

                            let before = &result[..lhs_start];
                            let after = &result[rhs_end..];
                            result = format!("{}{}{}", before, fixed_eq, after);
                            fixed_one = true;
                            break;
                        }
                    }
                }
            }

            if !fixed_one {
                break;
            }
        }

        (result, fixes)
    }

    fn eval_expr(&self, expr: &str) -> Option<f64> {
        let expr = expr.trim();
        if let Ok(v) = expr.parse::<f64>() {
            return Some(v);
        }

        // Find rightmost + or - (lowest precedence)
        if let Some(pos) = self.find_op(expr, &['+', '-']) {
            let left = self.eval_expr(&expr[..pos])?;
            let right = self.eval_expr(&expr[pos + 1..])?;
            return match expr.as_bytes()[pos] {
                b'+' => Some(left + right),
                b'-' => Some(left - right),
                _ => None,
            };
        }

        // Find rightmost * or / (higher precedence)
        if let Some(pos) = self.find_op(expr, &['*', '/']) {
            let left = self.eval_expr(&expr[..pos])?;
            let right = self.eval_expr(&expr[pos + 1..])?;
            return match expr.as_bytes()[pos] {
                b'*' => Some(left * right),
                b'/' => {
                    if right == 0.0 {
                        None
                    } else {
                        Some(left / right)
                    }
                }
                _ => None,
            };
        }

        None
    }

    fn find_op(&self, expr: &str, ops: &[char]) -> Option<usize> {
        let bytes = expr.as_bytes();
        let mut depth = 0i32;
        for i in (0..bytes.len()).rev() {
            match bytes[i] {
                b')' | b']' => depth += 1,
                b'(' | b'[' => depth -= 1,
                _ if depth == 0 && ops.contains(&(bytes[i] as char)) && i > 0 => {
                    let prev = bytes[i - 1];
                    if prev == b'('
                        || prev == b'['
                        || prev == b'+'
                        || prev == b'-'
                        || prev == b'*'
                        || prev == b'/'
                        || prev == b'^'
                        || prev == b'='
                    {
                        continue;
                    }
                    return Some(i);
                }
                _ => {}
            }
        }
        None
    }

    fn fix_units(&self, text: &str, _context: &str) -> (String, Vec<TokenFix>) {
        let mut fixes = Vec::new();
        let mut result = text.to_string();

        let unit_conversions = [
            ("km", "miles", 0.621371),
            ("miles", "km", 1.60934),
            ("kg", "lbs", 2.20462),
            ("lbs", "kg", 0.453592),
        ];

        let mut replacements: Vec<(usize, usize, String, String, String)> = Vec::new();

        for (from_unit, to_unit, conversion) in &unit_conversions {
            let pattern = format!(r"(\d+)\s*{}", from_unit);
            let mut search_start = 0;
            while search_start < result.len() {
                let search_text = &result[search_start..];
                if let Some(captures) = self.simple_regex_match(&pattern, search_text) {
                    if captures.len() == 1 {
                        if let Ok(value) = captures[0].parse::<f64>() {
                            let converted = value * conversion;
                            let original = format!("{} {}", captures[0], from_unit);
                            let fixed = format!("{} {}", self.format_number(converted), to_unit);

                            if let Some(pos) = search_text.find(&original) {
                                let abs_start = search_start + pos;
                                let abs_end = abs_start + original.len();
                                let overlaps = replacements
                                    .iter()
                                    .any(|(s, e, _, _, _)| abs_start < *e && abs_end > *s);
                                if !overlaps {
                                    replacements.push((
                                        abs_start,
                                        abs_end,
                                        original.clone(),
                                        fixed,
                                        format!("Unit conversion: {} to {}", from_unit, to_unit),
                                    ));
                                }
                                search_start += original.len();
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        replacements.sort_by_key(|x| std::cmp::Reverse(x.0));

        for (_, _, original, fixed, reason) in &replacements {
            let token_fix = TokenFix::new(original, fixed, reason, 0.9);
            fixes.push(token_fix);
            result = result.replacen(original, fixed, 1);
        }

        (result, fixes)
    }

    fn fix_typos(&self, text: &str) -> (String, Vec<TokenFix>) {
        let mut fixes = Vec::new();
        let mut result = text.to_string();

        let typos = [
            ("teh", "the"),
            ("hte", "the"),
            ("adn", "and"),
            ("taht", "that"),
            ("wiht", "with"),
            ("recieve", "receive"),
            ("reciever", "receiver"),
            ("recieved", "received"),
            ("occured", "occurred"),
            ("occurence", "occurrence"),
            ("occuring", "occurring"),
            ("seperate", "separate"),
            ("seperately", "separately"),
            ("seperation", "separation"),
            ("definately", "definitely"),
            ("definatly", "definitely"),
            ("definetly", "definitely"),
            ("definitaly", "definitely"),
            ("accomodate", "accommodate"),
            ("accomodation", "accommodation"),
            ("accomodated", "accommodated"),
            ("occassion", "occasion"),
            ("occassionally", "occasionally"),
            ("neccessary", "necessary"),
            ("neccesary", "necessary"),
            ("necessery", "necessary"),
            ("untill", "until"),
            ("becuase", "because"),
            ("becasue", "because"),
            ("beleive", "believe"),
            ("beleived", "believed"),
            ("beleiver", "believer"),
            ("wierd", "weird"),
            ("thier", "their"),
            ("goverment", "government"),
            ("governement", "government"),
            ("enviroment", "environment"),
            ("enviromental", "environmental"),
            ("enviroments", "environments"),
            ("independant", "independent"),
            ("independance", "independence"),
            ("consistant", "consistent"),
            ("consistancy", "consistency"),
            ("persistant", "persistent"),
            ("persistancy", "persistence"),
            ("maintainance", "maintenance"),
            ("maintenence", "maintenance"),
            ("maintainence", "maintenance"),
            ("arguement", "argument"),
            ("arguements", "arguments"),
            ("calender", "calendar"),
            ("calenders", "calendars"),
            ("weild", "wield"),
            ("concensus", "consensus"),
            ("preceeding", "preceding"),
            ("preceed", "precede"),
            ("proffesional", "professional"),
            ("proffession", "profession"),
            ("proffessor", "professor"),
            ("succesful", "successful"),
            ("successfull", "successful"),
            ("succesfully", "successfully"),
            ("successully", "successfully"),
            ("transfered", "transferred"),
            ("transfering", "transferring"),
            ("refered", "referred"),
            ("refering", "referring"),
            ("prefered", "preferred"),
            ("prefering", "preferring"),
            ("begining", "beginning"),
            ("runing", "running"),
            ("planing", "planning"),
            ("occuring", "occurring"),
            ("comming", "coming"),
            ("commited", "committed"),
            ("commiting", "committing"),
            ("formated", "formatted"),
            ("formating", "formatting"),
            ("targetting", "targeting"),
            ("targetted", "targeted"),
            ("writeable", "writable"),
            ("acheive", "achieve"),
            ("acheived", "achieved"),
            ("acheiving", "achieving"),
            ("aknowledge", "acknowledge"),
            ("aknowledged", "acknowledged"),
            ("apparantly", "apparently"),
            ("apparrently", "apparently"),
            ("assasination", "assassination"),
            ("basicly", "basically"),
            ("basicallly", "basically"),
            ("boundry", "boundary"),
            ("boundries", "boundaries"),
            ("camoflage", "camouflage"),
            ("catagory", "category"),
            ("catagories", "categories"),
            ("certainly", "certainly"),
            ("changable", "changeable"),
            ("charachter", "character"),
            ("charcter", "character"),
            ("collectable", "collectible"),
            ("comming", "coming"),
            ("commited", "committed"),
            ("comparision", "comparison"),
            ("compatability", "compatibility"),
            ("compatablity", "compatibility"),
            ("competance", "competence"),
            ("competant", "competent"),
            ("concatinate", "concatenate"),
            ("concatination", "concatenation"),
            ("concensus", "consensus"),
            ("concious", "conscious"),
            ("conciousness", "consciousness"),
            ("consistant", "consistent"),
            ("containes", "contains"),
            ("contians", "contains"),
            ("convienence", "convenience"),
            ("convienient", "convenient"),
            ("copywrite", "copyright"),
            ("definately", "definitely"),
            ("dependant", "dependent"),
            ("dilema", "dilemma"),
            ("dissapear", "disappear"),
            ("dissapoint", "disappoint"),
            ("dissapointed", "disappointed"),
            ("dissapointment", "disappointment"),
            ("doesnt", "doesn't"),
            ("dont", "don't"),
            ("embarass", "embarrass"),
            ("embarassed", "embarrassed"),
            ("embarassing", "embarrassing"),
            ("enviroment", "environment"),
            ("equiptment", "equipment"),
            ("existance", "existence"),
            ("existant", "existent"),
            ("explaination", "explanation"),
            ("familar", "familiar"),
            ("finaly", "finally"),
            ("foriegn", "foreign"),
            ("freind", "friend"),
            ("freindly", "friendly"),
            ("freinds", "friends"),
            ("functin", "function"),
            ("futher", "further"),
            ("goverment", "government"),
            ("grammer", "grammar"),
            ("happend", "happened"),
            ("happning", "happening"),
            ("harrass", "harass"),
            ("harrassed", "harassed"),
            ("harrassment", "harassment"),
            ("heirarchy", "hierarchy"),
            ("hygeine", "hygiene"),
            ("immediatly", "immediately"),
            ("immeditely", "immediately"),
            ("independant", "independent"),
            ("innoculate", "inoculate"),
            ("intresting", "interesting"),
            ("irregardless", "regardless"),
            ("knowlege", "knowledge"),
            ("knowledgable", "knowledgeable"),
            ("labratory", "laboratory"),
            ("liesure", "leisure"),
            ("lenght", "length"),
            ("liason", "liaison"),
            ("maintainance", "maintenance"),
            ("manuever", "maneuver"),
            ("milage", "mileage"),
            ("milisecond", "millisecond"),
            ("miniscule", "minuscule"),
            ("mischevous", "mischievous"),
            ("mispell", "misspell"),
            ("mispelling", "misspelling"),
            ("messeges", "messages"),
            ("messege", "message"),
            ("neccessary", "necessary"),
            ("necessery", "necessary"),
            ("noticable", "noticeable"),
            ("occassion", "occasion"),
            ("occassionally", "occasionally"),
            ("occurance", "occurrence"),
            ("occurence", "occurrence"),
            ("occurrance", "occurrence"),
            ("occuring", "occurring"),
            ("parliment", "parliament"),
            ("peice", "piece"),
            ("peices", "pieces"),
            ("posession", "possession"),
            ("posessing", "possessing"),
            ("prefered", "preferred"),
            ("prefering", "preferring"),
            ("preceeding", "preceding"),
            ("privelege", "privilege"),
            ("priveleged", "privileged"),
            ("proffesional", "professional"),
            ("professer", "professor"),
            ("publically", "publicly"),
            ("quarentine", "quarantine"),
            ("realy", "really"),
            ("reccommend", "recommend"),
            ("recomend", "recommend"),
            ("recomended", "recommended"),
            ("refered", "referred"),
            ("refering", "referring"),
            ("relavant", "relevant"),
            ("reluctent", "reluctant"),
            ("repitition", "repetition"),
            ("resistence", "resistance"),
            ("responsable", "responsible"),
            ("responsibilty", "responsibility"),
            ("seperate", "separate"),
            ("seperated", "separated"),
            ("seperately", "separately"),
            ("seperation", "separation"),
            ("seperatly", "separately"),
            ("similiar", "similar"),
            ("succesful", "successful"),
            ("succesfully", "successfully"),
            ("successfull", "successful"),
            ("supposedly", "supposedly"),
            ("supress", "suppress"),
            ("supressed", "suppressed"),
            ("thier", "their"),
            ("truely", "truly"),
            ("tyrany", "tyranny"),
            ("underate", "underrate"),
            ("undoubtably", "undoubtedly"),
            ("unfortunatly", "unfortunately"),
            ("unfortunatley", "unfortunately"),
            ("unfortunently", "unfortunately"),
            ("unneccesary", "unnecessary"),
            ("unneccessary", "unnecessary"),
            ("unnessary", "unnecessary"),
            ("untill", "until"),
            ("wierd", "weird"),
            ("whereever", "wherever"),
            ("wich", "which"),
            ("writting", "writing"),
            ("youre", "you're"),
            ("your welcome", "you're welcome"),
        ];

        for (typo, correction) in typos {
            let words: Vec<&str> = result.split_whitespace().collect();
            let mut new_words = Vec::new();
            let mut changed = false;

            for word in &words {
                let lower = word.to_lowercase();
                if lower == *typo {
                    let fixed = if word
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    {
                        let mut chars = correction.chars();
                        if let Some(first) = chars.next() {
                            format!("{}{}", first.to_uppercase(), chars.collect::<String>())
                        } else {
                            correction.to_string()
                        }
                    } else {
                        correction.to_string()
                    };

                    let token_fix = TokenFix::new(word, &fixed, "Typo correction", 0.9);
                    fixes.push(token_fix);
                    new_words.push(fixed);
                    changed = true;
                } else {
                    new_words.push(word.to_string());
                }
            }

            if changed {
                result = new_words.join(" ");
            }
        }

        (result, fixes)
    }

    fn fix_consistency(&self, text: &str, context: &str) -> (String, Vec<TokenFix>) {
        let mut fixes = Vec::new();
        let mut result = text.to_string();

        let context_lower = context.to_lowercase();
        let result_lower = result.to_lowercase();

        if context_lower.contains("python")
            && result_lower.contains("def ")
            && !result_lower.contains("def main()")
            && result_lower.contains("def ")
        {
            let token_fix = TokenFix::new(
                "",
                "Consider adding a main() function",
                "Python consistency",
                0.7,
            );
            fixes.push(token_fix);
        }

        if (context_lower.contains("javascript") || context_lower.contains("typescript"))
            && result_lower.contains("var ")
        {
            let token_fix = TokenFix::new(
                "var",
                "let/const",
                "Modern JS prefers let/const over var",
                0.8,
            );
            fixes.push(token_fix);
            result = result.replace("var ", "let ");
        }

        (result, fixes)
    }

    fn simple_regex_match(&self, pattern: &str, input: &str) -> Option<Vec<String>> {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let input_chars: Vec<char> = input.chars().collect();
        let mut matches = Vec::new();
        let mut i = 0;

        while i < input_chars.len() {
            if let Some(captures) = self.try_match_at(&pattern_chars, &input_chars, i) {
                matches = captures;
                break;
            }
            i += 1;
        }

        if matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }

    fn try_match_at(&self, pattern: &[char], input: &[char], start: usize) -> Option<Vec<String>> {
        let mut captures = Vec::new();
        let mut pi = 0;
        let mut ii = start;
        let mut current_capture = String::new();
        let mut in_capture = false;
        let mut last_char_type = ' ';

        while pi < pattern.len() {
            if ii >= input.len() {
                match pattern[pi] {
                    ')' => {
                        in_capture = false;
                        captures.push(current_capture.clone());
                        pi += 1;
                    }
                    '+' | '*' => {
                        pi += 1;
                    }
                    _ => return None,
                }
                continue;
            }
            match pattern[pi] {
                '(' => {
                    in_capture = true;
                    current_capture.clear();
                    pi += 1;
                }
                ')' => {
                    in_capture = false;
                    captures.push(current_capture.clone());
                    pi += 1;
                }
                '\\' => {
                    pi += 1;
                    if pi < pattern.len() {
                        if pattern[pi] == 'd' {
                            if input[ii].is_ascii_digit() {
                                if in_capture {
                                    current_capture.push(input[ii]);
                                }
                                ii += 1;
                                last_char_type = 'd';
                            } else {
                                let next = pi + 1;
                                if next < pattern.len()
                                    && (pattern[next] == '*' || pattern[next] == '+')
                                {
                                    last_char_type = 'd';
                                } else {
                                    return None;
                                }
                            }
                        } else if pattern[pi] == 's' {
                            if input[ii].is_ascii_whitespace() {
                                if in_capture {
                                    current_capture.push(input[ii]);
                                }
                                ii += 1;
                                last_char_type = 's';
                            } else {
                                let next = pi + 1;
                                if next < pattern.len()
                                    && (pattern[next] == '*' || pattern[next] == '+')
                                {
                                    last_char_type = 's';
                                } else {
                                    return None;
                                }
                            }
                        } else if input[ii] == pattern[pi] {
                            if in_capture {
                                current_capture.push(input[ii]);
                            }
                            ii += 1;
                            last_char_type = 'l';
                        } else {
                            return None;
                        }
                        pi += 1;
                    }
                }
                '+' => {
                    pi += 1;
                    while ii < input.len() {
                        match last_char_type {
                            'd' => {
                                if input[ii].is_ascii_digit() {
                                    if in_capture {
                                        current_capture.push(input[ii]);
                                    }
                                    ii += 1;
                                } else {
                                    break;
                                }
                            }
                            's' => {
                                if input[ii].is_ascii_whitespace() {
                                    if in_capture {
                                        current_capture.push(input[ii]);
                                    }
                                    ii += 1;
                                } else {
                                    break;
                                }
                            }
                            _ => break,
                        }
                    }
                }
                '*' => {
                    pi += 1;
                    while ii < input.len() {
                        match last_char_type {
                            's' => {
                                if input[ii].is_ascii_whitespace() {
                                    if in_capture {
                                        current_capture.push(input[ii]);
                                    }
                                    ii += 1;
                                } else {
                                    break;
                                }
                            }
                            _ => break,
                        }
                    }
                }
                '.' => {
                    if in_capture {
                        current_capture.push(input[ii]);
                    }
                    ii += 1;
                    pi += 1;
                }
                c if c == input[ii] => {
                    if in_capture {
                        current_capture.push(input[ii]);
                    }
                    ii += 1;
                    pi += 1;
                }
                _ => {
                    return None;
                }
            }
        }

        if pi == pattern.len() {
            Some(captures)
        } else {
            None
        }
    }

    fn format_number(&self, n: f64) -> String {
        if n == (n as i64) as f64 && n.abs() < 1e15 {
            format!("{}", n as i64)
        } else {
            format!("{:.2}", n)
        }
    }
}

impl Default for TokenFixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_math() {
        let fixer = TokenFixer::new();
        let (fixed, fixes) = fixer.fix("2 + 3 = 6", "math");
        assert_eq!(fixed, "2 + 3 = 5");
        assert_eq!(fixes.len(), 1);
        assert_eq!(fixes[0].original, "2 + 3 = 6");
        assert_eq!(fixes[0].fixed, "2 + 3 = 5");
    }

    #[test]
    fn test_fix_typos() {
        let fixer = TokenFixer::new();
        let (fixed, fixes) = fixer.fix("hte cat sat on teh mat", "");
        assert_eq!(fixed, "the cat sat on the mat");
        assert_eq!(fixes.len(), 2);
    }

    #[test]
    fn test_fix_consistency() {
        let fixer = TokenFixer::new();
        let (fixed, fixes) = fixer.fix("var x = 5", "javascript code");
        assert!(fixed.contains("let x = 5"));
        assert!(!fixes.is_empty());
    }
}

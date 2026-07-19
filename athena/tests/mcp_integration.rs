//! Integration tests for the MCP server entity and budget tools
//!
//! Tests the JSON-RPC handler layer directly by constructing AthenaRequest
//! objects and checking AthenaResponse from handle_request().

#[cfg(feature = "budget")]
use athena::budget::TokenBudget;
use athena::entity::{EntityRegistry, SeedEntity};
use athena::formula::{Formula, FormulaRegistry};
use athena::mcp::{AthenaMCP, AthenaRequest, AthenaResponse};
use athena::wheel::Domain;
use std::collections::HashMap;

/// Build a minimal MCP server with test seed entities.
fn setup_mcp() -> AthenaMCP {
    let mut registry = FormulaRegistry::new();
    registry
        .register_all(vec![Formula::atomic(
            "newtons_second",
            Domain::Shukra,
            vec!["mass", "acceleration"],
            "force",
            "mass * acceleration",
            "F = ma",
        )])
        .unwrap();

    let mut entities = EntityRegistry::new();
    entities.register_seed(SeedEntity {
        id: "test_entity_a".into(),
        name: "Test Entity A".into(),
        description: "Test entity on Pisces".into(),
        tags: vec!["test".into(), "pisces".into()],
        properties: HashMap::from([("arousal".into(), 0.8), ("valence".into(), -0.3)]),
        ..Default::default()
    });
    entities.register_seed(SeedEntity {
        id: "test_entity_b".into(),
        name: "Test Entity B".into(),
        description: "Test entity on Taurus".into(),
        tags: vec!["test".into(), "taurus".into()],
        properties: HashMap::from([("mass".into(), 5.0), ("acceleration".into(), 9.8)]),
        ..Default::default()
    });

    #[cfg(feature = "budget")]
    {
        let budget = TokenBudget::new(2000, 1000, 3000);
        AthenaMCP::with_context(registry, entities, budget)
    }
    #[cfg(not(feature = "budget"))]
    {
        AthenaMCP::with_entities(registry, entities)
    }
}

fn make_request(mcp: &AthenaMCP, method: &str, params: serde_json::Value) -> AthenaResponse {
    mcp.handle_request(AthenaRequest {
        method: method.to_string(),
        params,
    })
}

// ─── Entity Tools ─────────────────────────────────────────────────────────

#[test]
fn test_mcp_entity_list_returns_all_entities() {
    let mcp = setup_mcp();
    let response = make_request(&mcp, "entity_list", serde_json::json!({}));

    assert!(
        response.success,
        "entity_list should succeed: {:?}",
        response.error
    );
    assert!(
        response.data["count"].as_u64().unwrap_or(0) >= 2,
        "should have at least 2 entities"
    );
    assert!(response.data["entities"].is_array());
}

#[test]
fn test_mcp_entity_get_by_id() {
    let mcp = setup_mcp();
    let response = make_request(
        &mcp,
        "entity_get",
        serde_json::json!({"id": "test_entity_a"}),
    );

    assert!(response.success);
    assert_eq!(response.data["entity"]["id"], "test_entity_a");
    // Seed entities return "kind": "seed" not domain
    assert_eq!(response.data["entity"]["kind"], "seed");
    assert!(response.data["entity"]["properties"].is_object());
}

#[test]
fn test_mcp_entity_get_nonexistent() {
    let mcp = setup_mcp();
    let response = make_request(&mcp, "entity_get", serde_json::json!({"id": "nonexistent"}));

    assert!(!response.success, "should fail for nonexistent entity");
    assert!(response.error.unwrap().contains("not found"));
}

#[test]
fn test_mcp_entity_aspect_between_known() {
    // Setup: create an entity registry with runtime tokens
    use athena::astrology::ChangeSorter;
    use athena::entity::EntityRegistry;
    use athena::formula::FormulaRegistry;
    use athena::mcp::AthenaMCP;

    let registry = FormulaRegistry::new();
    let mut entities = EntityRegistry::new();
    let sorter = ChangeSorter::new();

    // Record two runtime tokens (creates entities with classification)
    let ids = entities.record_tokens(&["force", "mass"], &sorter);
    assert_eq!(ids.len(), 2);

    // Create MCP handler with the pre-populated registry
    let mcp = AthenaMCP::with_entities(registry, entities);

    // Test aspect between the two runtime entities
    let response = make_request(
        &mcp,
        "entity_aspect",
        serde_json::json!({"from": ids[0], "to": ids[1]}),
    );

    assert!(
        response.success,
        "entity_aspect should succeed: {:?}",
        response.error
    );
    assert!(
        response.data["aspect"].is_string(),
        "aspect should be a string"
    );
    assert!(
        response.data["entity_a"].is_string(),
        "entity_a should be present"
    );
    assert!(
        response.data["entity_b"].is_string(),
        "entity_b should be present"
    );
    assert!(
        response.data["arc_distance"].is_number(),
        "arc_distance should be present"
    );
}

#[test]
fn test_mcp_entity_search_by_tag() {
    let mcp = setup_mcp();
    let response = make_request(
        &mcp,
        "entity_search",
        serde_json::json!({"keyword": "pisces"}),
    );

    assert!(response.success);
    let count = response.data["count"].as_u64().unwrap_or(0);
    assert!(
        count >= 1,
        "should find at least 1 entity with 'pisces' tag"
    );
}

#[test]
fn test_mcp_entity_search_by_id() {
    let mcp = setup_mcp();
    let response = make_request(
        &mcp,
        "entity_search",
        serde_json::json!({"keyword": "test_entity_b"}),
    );

    assert!(response.success);
    assert_eq!(response.data["count"], 1);
    assert_eq!(response.data["entities"][0]["id"], "test_entity_b");
}

#[test]
fn test_mcp_entity_search_no_results() {
    let mcp = setup_mcp();
    let response = make_request(
        &mcp,
        "entity_search",
        serde_json::json!({"keyword": "zzz_nonexistent_zzz"}),
    );

    assert!(response.success);
    assert_eq!(response.data["count"], 0);
}

// ─── Budget Tools ──────────────────────────────────────────────────────────

#[cfg(feature = "budget")]
#[test]
fn test_mcp_budget_stats_initial() {
    let mcp = setup_mcp();
    let response = make_request(&mcp, "budget_stats", serde_json::json!({}));

    assert!(response.success);
    // Initial budget: 0 tokens used
    assert_eq!(response.data["prompt_used"], 0);
    assert_eq!(response.data["completion_used"], 0);
    assert_eq!(response.data["total_used"], 0);
    assert_eq!(response.data["exceeded"], false);
}

#[cfg(feature = "budget")]
#[test]
fn test_mcp_budget_stats_shows_limits() {
    let mcp = setup_mcp();
    let response = make_request(&mcp, "budget_stats", serde_json::json!({}));

    assert!(response.success);
    assert_eq!(response.data["max_prompt"], 2000);
    assert_eq!(response.data["max_completion"], 1000);
    assert_eq!(response.data["max_total"], 3000);
}

// ─── Tool Listing ─────────────────────────────────────────────────────────

#[test]
fn test_mcp_tool_list_includes_new_tools() {
    let mcp = setup_mcp();
    let response = make_request(&mcp, "tools", serde_json::json!({}));

    assert!(response.success);
    let tools = response.data["tools"].as_array().unwrap();
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(
        tool_names.contains(&"athena_entity_list"),
        "tools should include entity_list: {:?}",
        tool_names
    );
    assert!(
        tool_names.contains(&"athena_entity_get"),
        "tools should include entity_get: {:?}",
        tool_names
    );
    assert!(
        tool_names.contains(&"athena_entity_aspect"),
        "tools should include entity_aspect: {:?}",
        tool_names
    );
    assert!(
        tool_names.contains(&"athena_entity_search"),
        "tools should include entity_search: {:?}",
        tool_names
    );
    #[cfg(feature = "budget")]
    {
        assert!(
            tool_names.contains(&"athena_budget_stats"),
            "tools should include budget_stats: {:?}",
            tool_names
        );
        assert!(
            tool_names.contains(&"athena_budget_reset"),
            "tools should include budget_reset: {:?}",
            tool_names
        );
    }
}

// ─── Compaction & Token Savings (wire layer) ──────────────────────────────

/// Build a handler with enough entities to trip the default list cap.
fn setup_handler_with_many_entities() -> athena::mcp::McpHandler {
    let mut registry = FormulaRegistry::new();
    registry
        .register_all(vec![Formula::atomic(
            "newtons_second",
            Domain::Shukra,
            vec!["mass", "acceleration"],
            "force",
            "mass * acceleration",
            "F = ma",
        )])
        .unwrap();

    let mut entities = EntityRegistry::new();
    for i in 0..30 {
        entities.register_seed(SeedEntity {
            id: format!("bulk_entity_{i:02}"),
            name: format!("Bulk Entity {i}"),
            description: "Filler entity — cosmological flavor text".into(),
            tags: vec!["bulk".into()],
            properties: HashMap::from([("mass".into(), i as f64)]),
            ..Default::default()
        });
    }
    athena::mcp::McpHandler::new(AthenaMCP::with_entities(registry, entities))
}

/// Extract the text payload of a CallToolResult via its JSON form.
fn tool_result_json(result: &rmcp::model::CallToolResult) -> serde_json::Value {
    let raw = serde_json::to_value(result).expect("CallToolResult serializes");
    let text = raw["content"][0]["text"]
        .as_str()
        .expect("tool result has text content");
    serde_json::from_str(text).expect("tool result text is JSON")
}

#[test]
fn test_wire_entity_list_compacted_by_default() {
    let handler = setup_handler_with_many_entities();
    let result = handler.call_tool_sync("athena_entity_list", None);
    let data = tool_result_json(&result);

    let shown = data["entities"].as_array().unwrap();
    assert_eq!(shown.len(), 25, "default limit caps the list");
    assert_eq!(data["omitted"], 5, "omission is reported");
    assert_eq!(data["count"], 30, "total count preserved");
    assert!(
        shown[0].get("properties").is_none(),
        "compact entries drop properties"
    );
    assert!(shown[0].get("id").is_some() && shown[0].get("name").is_some());
}

#[test]
fn test_wire_detail_full_bypasses_compaction() {
    let handler = setup_handler_with_many_entities();
    let args: serde_json::Map<String, serde_json::Value> =
        serde_json::from_value(serde_json::json!({"detail": "full"})).unwrap();
    let result = handler.call_tool_sync("athena_entity_list", Some(&args));
    let data = tool_result_json(&result);

    let shown = data["entities"].as_array().unwrap();
    assert_eq!(shown.len(), 30, "full detail returns everything");
    assert!(data.get("omitted").is_none());
    assert!(
        shown[0].get("properties").is_some(),
        "full entries keep properties"
    );
}

#[test]
fn test_wire_limit_param_respected() {
    let handler = setup_handler_with_many_entities();
    let args: serde_json::Map<String, serde_json::Value> =
        serde_json::from_value(serde_json::json!({"limit": 5})).unwrap();
    let result = handler.call_tool_sync("athena_entity_list", Some(&args));
    let data = tool_result_json(&result);

    assert_eq!(data["entities"].as_array().unwrap().len(), 5);
    assert_eq!(data["omitted"], 25);
}

#[test]
fn test_wire_savings_ledger_accumulates() {
    let handler = setup_handler_with_many_entities();
    handler.call_tool_sync("athena_entity_list", None);
    handler.call_tool_sync("athena_formula_search", {
        let args: serde_json::Map<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({"keyword": "newtons"})).unwrap();
        Some(&args.clone())
    });

    let result = handler.call_tool_sync("athena_savings", None);
    let data = tool_result_json(&result);
    assert!(data["calls"].as_u64().unwrap() >= 2);
    assert!(
        data["saved_tokens"].as_u64().unwrap() > 0,
        "compaction saved tokens: {data}"
    );
    assert!(data["saved_pct"].as_f64().unwrap() > 0.0);
}

#[test]
fn test_savings_method_initially_zero() {
    let mcp = setup_mcp();
    let response = make_request(&mcp, "savings", serde_json::json!({}));
    assert!(response.success);
    assert_eq!(response.data["calls"], 0);
    assert_eq!(response.data["saved_tokens"], 0);
}

pub struct McpServer;

impl McpServer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_server_creates() {
        let _s = McpServer::new();
    }
}

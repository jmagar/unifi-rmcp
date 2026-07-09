# unifi-rmcp

Node launcher for the `runifi` Rust MCP server and CLI binary.

```bash
npx -y unifi-rmcp --help
```

The package downloads the matching GitHub Release binary during `postinstall`.

## MCP stdio

Use the package directly as an MCP command:

```json
{
  "mcpServers": {
    "unifi": {
      "command": "npx",
      "args": ["-y", "unifi-rmcp"]
    }
  }
}
```

## Environment

- `UNIFI_RMCP_BINARY_VERSION`: release tag/version to download, defaulting to this npm package version.
- `UNIFI_RMCP_VERSION`: alias for `UNIFI_RMCP_BINARY_VERSION`.
- `UNIFI_RMCP_REPO`: GitHub `owner/repo`, defaulting to `jmagar/unifi-rmcp`.
- `UNIFI_RMCP_RELEASE_BASE_URL`: full release download base URL.
- `UNIFI_RMCP_SKIP_DOWNLOAD=1`: skip postinstall download.

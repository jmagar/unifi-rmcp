# UniFi API Coverage

## Sources

- Official Network API: `data/unifi_official_network_v10_3_58.json`
- Internal capability reference inventory: `data/unifi_internal_reference_tools.json`

## API Families

- `official`: documented Network Integration API under `/proxy/network/integration/v1`.
- `internal`: Network controller APIs under `/proxy/network/api/s/{site}` and `/proxy/network/v2/api/site/{site}`.
- `hybrid`: convenience actions that use internal actions by default and switch to official API when `siteId` or `prefer="official"` is supplied.

## Initial Coverage

- Official Network operations targeted: 78.
- Internal Network reference rows captured: 180; live-verified runtime capabilities: 12.
- Existing live-verified rustifi actions preserved: clients, devices, wlans, health, alarms, sysinfo, me.

## Implementation Status

| Action | Family | Endpoint | Status |
|---|---|---|---|
| `official_*` | official | `/proxy/network/integration/v1/...` | implemented by generic dispatcher |
| `clients` | internal | `GET /stat/sta` | preserved |
| `devices` | internal | `GET /stat/device` | preserved |
| `wlans` | internal | `GET /rest/wlanconf` | preserved |
| `health` | internal | `GET /stat/health` | preserved |
| `alarms` | internal | `GET /rest/alarm` | preserved |
| `sysinfo` | internal | `GET /stat/sysinfo` | preserved |
| `me` | internal | `GET /proxy/network/api/self` | preserved |
| `internal_list_alarms` | internal | `GET /rest/alarm` | generic internal dispatcher |
| `internal_get_network_health` | internal | `GET /stat/health` | generic internal dispatcher |
| `internal_list_networks` | internal | `GET /rest/networkconf` | generic internal dispatcher |
| `internal_list_port_forwards` | internal | `GET /rest/portforward` | generic internal dispatcher |
| `internal_trigger_rf_scan` | internal | `POST /cmd/devmgr` | admin-authorized generic dispatcher |
| `list_clients` | hybrid | official clients or `clients` | implemented |
| `list_devices` | hybrid | official devices or `devices` | implemented |
| `list_networks` | hybrid | official networks or `internal_list_networks` | implemented |
| `list_wifi` | hybrid | official WiFi or `wlans` | implemented |
| `get_system_info` | hybrid | official info or `sysinfo` | implemented |

The internal reference inventory is registry-backed, but only verified internal rows are exposed as runtime MCP/CLI actions. Existing and explicitly mapped internal actions use known controller endpoints; the broader 180-row reference remains a research catalog until each path is verified against live controller behavior.

## Endpoint Verification

Run live route probes against a controller with:

```bash
UNIFI_URL=https://<gateway> \
UNIFI_API_KEY=<network-api-key> \
UNIFI_SITE=default \
UNIFI_SITE_ID=<official-site-uuid> \
UNIFI_SKIP_TLS_VERIFY=true \
cargo run -p xtask -- verify-api-endpoints
```

The verifier writes `data/unifi_endpoint_verification_report.json`.

Result interpretation:

- `ok`: endpoint returned a 2xx response.
- `route_reached_rejected_probe`: endpoint returned an expected 4xx for an inert or placeholder probe; this usually means the route/auth/path reached controller validation.
- `auth_or_permission_failed`: API key was rejected or lacks permission.
- `server_error`: request failed or controller returned 5xx.
- `missing_site_id`: set `UNIFI_SITE_ID` to probe site-scoped official endpoints.

By default `UNIFI_VERIFY_MUTATING=true` sends inert bodies and placeholder object IDs to mutating endpoints. Set `UNIFI_VERIFY_MUTATING=false` to probe only read endpoints. Set `UNIFI_VERIFY_UNVERIFIED_INTERNAL=false` to skip unverified internal reference rows.

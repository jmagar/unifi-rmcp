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
- Internal Network reference rows captured: 180; verified runtime capabilities: 16.
- Existing rustifi actions preserved: clients, devices, wlans, health, alarms, events, sysinfo, me.

## Implementation Status

| Action | Family | Endpoint | Status |
|---|---|---|---|
| `official_*` | official | `/proxy/network/integration/v1/...` | implemented by generic dispatcher |
| `clients` | internal | `GET /stat/sta` | preserved |
| `devices` | internal | `GET /stat/device` | preserved |
| `wlans` | internal | `GET /rest/wlanconf` | preserved |
| `health` | internal | `GET /stat/health` | preserved |
| `alarms` | internal | `GET /rest/alarm` | preserved |
| `events` | internal | `GET /rest/event` | preserved |
| `sysinfo` | internal | `GET /stat/sysinfo` | preserved |
| `me` | internal | `GET /api/self` | preserved |
| `internal_list_alarms` | internal | `GET /rest/alarm` | generic internal dispatcher |
| `internal_list_events` | internal | `GET /rest/event` | generic internal dispatcher |
| `internal_get_network_health` | internal | `GET /stat/health` | generic internal dispatcher |
| `internal_list_networks` | internal | `GET /rest/networkconf` | generic internal dispatcher |
| `internal_list_port_forwards` | internal | `GET /rest/portforward` | generic internal dispatcher |
| `internal_list_dns_records` | internal | `GET /rest/dnsrecord` | generic internal dispatcher |
| `internal_get_switch_ports` | internal | `GET /stat/switch-port` | generic internal dispatcher |
| `internal_trigger_rf_scan` | internal | `POST /cmd/devmgr` | confirmation-gated generic dispatcher |
| `list_clients` | hybrid | official clients or `clients` | implemented |
| `list_devices` | hybrid | official devices or `devices` | implemented |
| `list_networks` | hybrid | official networks or `internal_list_networks` | implemented |
| `list_wifi` | hybrid | official WiFi or `wlans` | implemented |
| `get_system_info` | hybrid | official info or `sysinfo` | implemented |

The internal reference inventory is registry-backed, but only verified internal rows are exposed as runtime MCP/CLI actions. Existing and explicitly mapped internal actions use known controller endpoints; the broader 180-row reference remains a research catalog until each path is verified against live controller behavior.

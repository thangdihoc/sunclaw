# Sunclaw Architecture Overview

Sunclaw follows a layered architecture:

1. Core contracts (`sunclaw-core`)
2. Runtime engine (`sunclaw-runtime`)
3. Policy and sandbox (`sunclaw-policy`, future `sunclaw-sandbox`)
4. Skills and orchestration (`sunclaw-skills`, `sunclaw-orchestrator`)
5. Delivery channels (`sunclaw-cli`, future HTTP/Telegram adapters)

The intended flow is: inbound message -> orchestrator routing -> runtime decision -> policy gate -> tool/provider execution -> memory persistence -> outbound response.

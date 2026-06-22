## ADDED Requirements

### Requirement: Resilient Telemetry Delivery

The system SHALL fail silently when telemetry delivery is blocked, so that firewalled or offline environments never surface network errors to the user.

#### Scenario: Network blocked by firewall
- **WHEN** telemetry is enabled but the analytics endpoint is unreachable or blocked
- **THEN** system swallows the network error without printing it
- **AND** system does not retry indefinitely (delivery uses a short ~1s timeout with retries and remote config disabled)

#### Scenario: No impact on command exit
- **WHEN** a telemetry send fails
- **THEN** the command still completes and exits normally

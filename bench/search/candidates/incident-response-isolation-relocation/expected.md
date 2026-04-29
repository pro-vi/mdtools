# Incident Response Runbook

## Initial Triage

Use this section during the first 30 minutes of an incident.

### Gather Information

- Identify affected systems
- Determine severity and impact
- Notify stakeholders

### Open Incident Channel

- Create the incident room
- Pin the incident commander
- Start the timeline

## Containment

Use this section once the initial facts are known.

### Isolate Affected Systems

- Quarantine compromised hosts
- Disconnect from the network if needed
- Preserve evidence for forensics

> Do not power off systems until volatile evidence has been captured.

### Limit Spread

- Apply temporary patches
- Block malicious IPs
- Disable vulnerable services

### Isolate Affected Systems Checklist

This checklist is for post-containment verification and must remain in place.

| Check | Owner |
|---|---|
| Network quarantine reviewed | SRE |
| Access tokens revoked | Security |

### Eradicate Threat

- Remove malware
- Reset compromised credentials
- Patch vulnerabilities

```text
Legacy triage labels:
### Isolate Affected Systems
### Open Incident Channel
```

## Recovery

### Restore Services

- Bring systems back online
- Monitor for stability
- Validate data integrity

### Conduct Review

- Document lessons learned
- Update the runbook as needed
- Schedule follow-up training

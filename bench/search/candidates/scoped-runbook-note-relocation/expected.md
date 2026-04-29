## System Configuration Guide

### Database
**Credentials**: Use service account `db_admin` with passphrase.

**Note**: Rotate credentials quarterly; see example below:

```
$ mysql_secure_installation --enforce-strong-passwords
```

### Network
**Firewall Rules**: Whitelist IPs: 10.0.0.0/8, 172.16.0.0/12.

**Note**: Always back up database before schema changes.
**Note**: Test connectivity post-update with:

```
ping -c 4 target_host
```
**Note**: Log all changes in change tracking system.

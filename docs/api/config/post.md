
# Configure application

- Enable or disable dry run mode
- Change log level

**URL** : `/api/config`

**Method** : `POST`

### Request constraints

**Content-Type**: `application/json`

**Body**:

| Field       | Type     | Is required | Note                        |
|-------------|----------|-------------|-----------------------------|
| `dry_run`   | 'bool`   | False       | Enable/disable dry run mode | 
| `log_level` | 'string` | False       | Sets log level              | 

**Request examples**

```json
{
    "dry_run": true
}
```

```json
{
    "dry_run": false,
    "log_level": "error,firewall_cloudflare_executor=trace"
}
```

## Success Response

**Condition** : Successfully updated service configuration

**Code** : `204 NO CONTENT`

## Error Responses

**Condition** : Request is incorrect.

**Code** : `400 BAD REQUEST`

```json
{
    "code": 400,
    "reason": "Log level is incorrect"
}
```

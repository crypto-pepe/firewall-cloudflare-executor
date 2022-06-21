# Ban target

Bans target with provided details.

**URL** : `/api/bans`

**Method** : `POST`

### Request constraints

**Headers**:

- `X-Analyzer-Id`

**Content-Type**: `application/json`

| Field    | Type                                  | Is required | Note                |
|----------|---------------------------------------|-------------|---------------------|
| `target` | `{ ip: string?, user_agent:string? }` | Yes         | Ban target          | 
| `reason` | `string`                              | Yes         | Ban reason          |
| `ttl`    | `number`                              | Yes         | Ban TTL, in seconds |

**Request examples**

```json
    {
        "target": {
            "ip": "11.12.13.14",
            "user_agent": "curl user-agent"
        },
        "reason": "Exceeded requests per minute limit, excess 2.0",
        "ttl": 300
    }
```

```json
{
    "target": {
        "ip": "1.1.1.1"
    },
    "reason": "Exceeded requests per minute limit, excess 2.0",
    "ttl": 300
}
```

## Success Response

**Condition** : Ban was successfully applied.

**Code** : `204 NO CONTENT`

## Error Responses

**Condition** : If fields are missed.

**Code** : `400 BAD REQUEST`

**Body example**

```json
{
    "code": 400,
    "reason": "Provided request does not match the constraints",
    "details": {
        "target": "This field is required"
    }
}
```

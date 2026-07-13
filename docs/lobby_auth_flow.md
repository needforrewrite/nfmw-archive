# Get a key for the lobby

GET /auth/service-key?service=lobby
Authorization: Bearer <archive session token>

{ "serviceKey": "kP9x…", "expiresInSeconds": 120,
  "service": { "id": "lobby", "url": "http://localhost:7001" } }

# The lobby redeems it

POST /auth/service-key/validate
Authorization: NFMW-HMAC-SHA256 service=lobby,ts=1752436800,sig=a1b2…
Content-Type: application/json

{ "serviceKey": "kP9x…" }

{ "userId": 1, "username": "racer31262", "roles": [] }
Redeeming consumes the key, so a stolen one is worthless the moment the real player uses it. Roles ride in the response because the lobby has no database to look them up in. A lobby cannot redeem a key minted for a different service — the audience must match the service= that signed the request.

The signing string, which is what the C# side needs to reproduce:

{METHOD}\n{path+query}\n{serviceId}\n{unixTimestamp}\n{hex(sha256(body))}
Signed with HMAC-SHA256, hex-encoded. Method, path, query, target service, timestamp and body are all bound in, so a captured signature can't be moved to another route, another service, or another body.

# Managing service credentials
[[services]]
id = "lobby"
url = "http://localhost:7001"
hmac_secrets = ["<openssl rand -hex 32>"]

Rotation is a config edit, which is why hmac_secrets is a list: add the new secret, restart, move the lobby onto it, drop the old one. Both are accepted while both are listed, so there's no flag-day. Revocation is deleting the entry — that instantly kills both the lobby's ability to authenticate and the ability to mint keys for it. On boot the server refuses to start on a non-hex secret, a secret under 32 bytes, an empty secret list, or a service that tries to call itself archive.

The HMAC scheme authenticates but does not prevent replay inside the +/-30s clock-skew window. That's harmless for validate because redemption is single-use, but when you add /match-result later it must be idempotent on a match id.
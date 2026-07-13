Full OAuth flow:
 
1. Send GET to /auth/discord/start, returns { poll_id: string, url: string }
2. Client opens URL, authorizes with Discord. Callback is triggered on server, server performs token exchange and returns OK to Discord
3. Client polls /auth/poll/:poll_id using the poll_id in result of start, this returns { status: string, payload: string }
4. When exchange is incomplete, poll returns { status: "pending" }, wait until this changes
5. If status is "login", payload is the session token that can be used by the client
6. If status is "create_account", payload is a temp token used to register a username. POST /auth/discord/create_account with body { "temp_token": token, "username": username }
7. Returns { "username": username, "session_token": token }, use token for auth requests
 
Full local account creation flow:
 
1. Send POST to /auth/local/create_account with { username: "username", password: "password" } (will eventually require email too I guess)
2. Server validates, on success returns { username: "username" }
3. Send POST to /auth/local/login with same body
4. If login successful, returns { username: "username", "session_token": token }
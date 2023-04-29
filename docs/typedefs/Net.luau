type NetMethod = "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH"

--[=[
	@type NetFetchParams
	@within Net

	Parameters for sending network requests with `net.request`.

	This is a dictionary that may contain one or more of the following values:

	* `url` - The URL to send a request to. This is always required
	* `method` - The HTTP method verb, such as `"GET"`, `"POST"`, `"PATCH"`, `"PUT"`, or `"DELETE"`. Defaults to `"GET"`
	* `query` - A table of key-value pairs representing query parameters in the request path
	* `headers` - A table of key-value pairs representing headers
	* `body` - The request body
]=]
export type NetFetchParams = {
	url: string,
	method: NetMethod?,
	query: { [string]: string }?,
	headers: { [string]: string }?,
	body: string?,
}

--[=[
	@type NetFetchResponse
	@within new

	Response type for sending network requests with `net.request`.

	This is a dictionary containing the following values:

	* `ok` - If the status code is a canonical success status code, meaning within the range 200 -> 299
	* `statusCode` - The status code returned for the request
	* `statusMessage` - The canonical status message for the returned status code, such as `"Not Found"` for status code 404
	* `headers` - A table of key-value pairs representing headers
	* `body` - The request body, or an empty string if one was not given
]=]
export type NetFetchResponse = {
	ok: boolean,
	statusCode: number,
	statusMessage: string,
	headers: { [string]: string },
	body: string,
}

--[=[
	@type NetRequest
	@within Net

	Data type for requests in `net.serve`.

	This is a dictionary containing the following values:

	* `path` - The path being requested, relative to the root. Will be `/` if not specified
	* `query` - A table of key-value pairs representing query parameters in the request path
	* `method` - The HTTP method verb, such as `"GET"`, `"POST"`, `"PATCH"`, `"PUT"`, or `"DELETE"`. Will always be uppercase
	* `headers` - A table of key-value pairs representing headers
	* `body` - The request body, or an empty string if one was not given
]=]
export type NetRequest = {
	path: string,
	query: { [string]: string? },
	method: NetMethod,
	headers: { [string]: string },
	body: string,
}

--[=[
	@type NetRequest
	@within Net

	Response type for requests in `net.serve`.

	This is a dictionary that may contain one or more of the following values:

	* `status` - The status code for the request, in the range `100` -> `599`
	* `headers` - A table of key-value pairs representing headers
	* `body` - The response body
]=]
export type NetResponse = {
	status: number?,
	headers: { [string]: string }?,
	body: string?,
}

type NetServeHttpHandler = (request: NetRequest) -> string | NetResponse
type NetServeWebSocketHandler = (socket: NetWebSocket) -> ()

--[=[
	@type NetServeConfig
	@within Net

	Configuration for `net.serve`.

	This may contain one of, or both of the following callbacks:

	* `handleRequest` for handling normal http requests, equivalent to just passing a function to `net.serve`
	* `handleWebSocket` for handling web socket requests, which will receive a `NetWebSocket` object as its first and only parameter
]=]
export type NetServeConfig = {
	handleRequest: NetServeHttpHandler?,
	handleWebSocket: NetServeWebSocketHandler?,
}

--[=[
	@type NetServeHandle
	@within Net

	A handle to a currently running web server, containing a single `stop` function to gracefully shut down the web server.
]=]
export type NetServeHandle = {
	stop: () -> (),
}

--[=[
	@type NetWebSocket
	@within Net

	A reference to a web socket connection.

	The web socket may be in either an "open" or a "closed" state, changing its current behavior.

	When open:

	* Any function on the socket such as `send`, `next` or `close` can be called without erroring
	* `next` can be called to yield until the next message is received or the socket becomes closed

	When closed:

	* `next` will no longer return any message(s) and instead instantly return nil
	* `send` will throw an error stating that the socket has been closed

	Once the websocket has been closed, `closeCode` will no longer be nil, and will be populated with a close
	code according to the [WebSocket specification](https://www.iana.org/assignments/websocket/websocket.xhtml).
	This will be an integer between 1000 and 4999, where 1000 is the canonical code for normal, error-free closure.
]=]
export type NetWebSocket = {
	closeCode: number?,
	close: (code: number?) -> (),
	send: (message: string, asBinaryMessage: boolean?) -> (),
	next: () -> string?,
}

--[=[
	@class Net


	Built-in library for network access

	### Example usage

	```lua
	local net = require("@lune/net")

	-- Sending a web request
	local response = net.request("https://www.google.com")
	print(response.ok)
	print(response.statusCode, response.statusMessage)
	print(response.headers)

	-- Using a JSON web API
	local response = net.request({
		url = "https://dummyjson.com/products/add",
		method = "POST",
		headers = { ["Content-Type"] = "application/json" },
		body = net.jsonEncode({
			title = "Cool Pencil",
		})
	})
	local product = net.jsonDecode(response.body)
	print(product.id, "-", product.title)

	-- Starting up a webserver
	net.serve(8080, function(request)
		return {
			status = 200,
			body = "Echo:\n" .. request.body,
		}
	end)
	```
]=]
export type Net = {
	--[=[
		@within Net

		Sends an HTTP request using the given url and / or parameters, and returns a dictionary that describes the response received.

		Only throws an error if a miscellaneous network or I/O error occurs, never for unsuccessful status codes.

		@param config The URL or request config to use
		@return A dictionary representing the response for the request
	]=]
	request: (config: string | NetFetchParams) -> NetFetchResponse,
	--[=[
		@within Net
		@must_use

		Connects to a web socket at the given URL.

		Throws an error if the server at the given URL does not support
		web sockets, or if a miscellaneous network or I/O error occurs.

		@param url The URL to connect to
		@return A web socket handle
	]=]
	socket: (url: string) -> NetWebSocket,
	--[=[
		@within Net

		Creates an HTTP server that listens on the given `port`.

		This will ***not*** block and will keep listening for requests on the given `port`
		until the `stop` function on the returned `NetServeHandle` has been called.

		@param port The port to use for the server
		@param handlerOrConfig The handler function or config to use for the server
	]=]
	serve: (port: number, handlerOrConfig: NetServeHttpHandler | NetServeConfig) -> NetServeHandle,
	--[=[
		@within Net
		@must_use

		Encodes the given value as JSON.

		@param value The value to encode as JSON
		@param pretty If the encoded JSON string should include newlines and spaces. Defaults to false
		@return The encoded JSON string
	]=]
	jsonEncode: (value: any, pretty: boolean?) -> string,
	--[=[
		@within Net
		@must_use

		Decodes the given JSON string into a lua value.

		@param encoded The JSON string to decode
		@return The decoded lua value
	]=]
	jsonDecode: (encoded: string) -> any,
	--[=[
		@within Net
		@must_use

		Encodes the given string using URL encoding.

		@param s The string to encode
		@param binary If the string should be treated as binary data and/or is not valid utf-8. Defaults to false
		@return The encoded string
	]=]
	urlEncode: (s: string, binary: boolean?) -> string,
	--[=[
		@within Net
		@must_use

		Decodes the given string using URL decoding.

		@param s The string to decode
		@param binary If the string should be treated as binary data and/or is not valid utf-8. Defaults to false
		@return The decoded string
	]=]
	urlDecode: (s: string, binary: boolean?) -> string,
}

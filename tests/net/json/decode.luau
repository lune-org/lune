type Response = {
	products: {
		{
			id: number,
			title: string,
			description: string,
			price: number,
			discountPercentage: number,
			rating: number,
			stock: number,
			brand: string,
			category: string,
			thumbnail: string,
			images: { string },
		}
	},
	total: number,
	skip: number,
	limit: number,
}

local response = net.request("https://dummyjson.com/products")

assert(response.ok, "Dummy JSON api returned an error")
assert(#response.body > 0, "Dummy JSON api returned empty body")

local data: Response = net.jsonDecode(response.body)

assert(type(data.limit) == "number", "Products limit was not a number")
assert(type(data.products) == "table", "Products was not a table")
assert(#data.products > 0, "Products table was empty")

local productCount = 0
for _, product in data.products do
	productCount += 1
	assert(type(product.id) == "number", "Product id was not a number")
	assert(type(product.title) == "string", "Product title was not a number")
	assert(type(product.description) == "string", "Product description was not a number")
	assert(type(product.images) == "table", "Product images was not a table")
	assert(#product.images > 0, "Product images table was empty")
end

assert(
	data.limit == productCount,
	string.format(
		"Products limit and number of products in array mismatch (expected %d, got %d)",
		data.limit,
		productCount
	)
)

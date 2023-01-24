return function(index: number, type: string, value: any)
	if typeof(value) ~= type then
		console.error(
			string.format(
				"Expected argument #%d to be of type %s, got %s",
				index,
				type,
				console.format(value)
			)
		)
		process.exit(1)
	end
end

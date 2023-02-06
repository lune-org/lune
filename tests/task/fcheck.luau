return function(index: number, type: string, value: any)
	if typeof(value) ~= type then
		error(
			string.format(
				"Expected argument #%d to be of type %s, got %s",
				index,
				type,
				console.format(value)
			)
		)
	end
end

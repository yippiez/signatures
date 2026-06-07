MY_CONST: Annotated[int, Field(default=0)] = 0
FIELD: Annotated[str, Field(min_length=1, max_length=100)] = "default"
LITERAL_CONST: Literal["x=1"] = "x=1"

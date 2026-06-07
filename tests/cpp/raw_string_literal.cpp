void before();
const char* code = R"(
void fake() {}
{ { {
)";
const char* tagged = R"sql(
SELECT } ; FROM t
)sql";
void after();

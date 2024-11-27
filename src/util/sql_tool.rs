pub struct SqlTool {
    pub sql: String,
    pub opt_sql: Vec<String>,
    pub opt_val: Vec<rbs::Value>,
    pub condition_sql: String,
}

impl SqlTool {
    pub fn init(sql: &str, condition_sql: &str) -> Self {
        Self {
            sql: sql.to_string(),
            opt_sql: Vec::new(),
            opt_val: Vec::new(),
            condition_sql: condition_sql.to_string(),
        }
    }

    pub fn append_sql_filed(&mut self, filed: &str, value: rbs::Value) {
        self.opt_sql.push(format!(" {filed}=? "));
        self.opt_val.push(value);
    }
    fn gen_query_sql(&self) -> String {
        let sql = self.opt_sql.join(" and ");
        let where_str = {
            if self.opt_sql.is_empty() {
                ""
            } else {
                " where "
            }
        };

        format!("{where_str} {sql} ")
    }

    pub fn gen_sql(&self) -> String {
        let query_sql = self.gen_query_sql();
        let res = format!("{} {query_sql} {}", self.sql, self.condition_sql);
        res
    }

    pub fn gen_page_sql(&self, page_no: i32, take: i32) -> String {
        let query_sql = self.gen_query_sql();
        let offset = {
            if page_no < 0 {
                0
            } else {
                (page_no - 1) *take
            }
        };
        let res = format!(
            "{} {query_sql} {} limit {offset},{take}",
            self.sql, self.condition_sql,
        );
        res
    }

    pub fn gen_count_sql(&self, cont_sql: &str) -> String {
        let query_sql = self.gen_query_sql();
        let res = format!("{cont_sql} {query_sql}");
        res
    }
}
#[cfg(test)]
mod sql_tool_test {
    use crate::util::sql_tool::SqlTool;
    use rbs::to_value;
    #[test]
    fn get_gen_sql() {
        let mut tool = SqlTool::init("select * from user", "");
        tool.append_sql_filed("name", to_value!("123"));
        let sql = tool.gen_sql();
        println!("sql {}", sql);
    }
    #[test]
    fn count_sql() {
        let mut tool = SqlTool::init("select * from user", "");
        tool.append_sql_filed("name", to_value!("123"));
        let sql = tool.gen_count_sql("select count(1) from user");
        println!("{sql}");
    }
}

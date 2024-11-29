use rbatis::{executor::RBatisConnExecutor, Page};

pub struct SqlToolPageData<T> {
    pub ex_db: RBatisConnExecutor,
    pub table: String,
    pub records: Vec<T>,
    pub page_no: u64,
    pub page_size: u64,
}

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

    pub fn gen_page_sql(&self, page_no: i32, take: i32) -> String {
        let query_sql = self.gen_query_sql();
        let offset = {
            if page_no < 0 {
                0
            } else {
                (page_no - 1) * take
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

    pub async fn page_query<T: Sync + Send>(&self, data: SqlToolPageData<T>) -> Page<T> {
        let count_sql = self.gen_count_sql(&format!("select count(1) from {}", data.table));
        let total: u64 = data
            .ex_db
            .query_decode(&count_sql, self.opt_val.clone())
            .await
            .expect("msg");

        Page {
            records: data.records,
            total,
            page_no: data.page_no,
            page_size: data.page_size,
            do_count: true,
        }
    }
}
#[cfg(test)]
mod sql_tool_test {
    use crate::util::sql_tool::SqlTool;
    use rbs::to_value;
    #[test]
    fn count_sql() {
        let mut tool = SqlTool::init("select * from user", "");
        tool.append_sql_filed("name", to_value!("123"));
        let sql = tool.gen_count_sql("select count(1) from user");
        println!("{sql}");
    }
}

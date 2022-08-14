use crate::domain::table::SysTrash;
use crate::pool;
use rbatis::object_id::ObjectId;
use rbdc::datetime::FastDateTime;
use rbdc::db::ExecResult;
use rbdc::Error;
use serde::Serialize;

/// 一个垃圾桶服务，可以回收数据。找回数据，展示垃圾桶数据
pub struct SysTrashService {}

impl SysTrashService {
    pub async fn add<T>(&self, table_name: &str, args: &[T]) -> Result<u64, Error>
    where
        T: Serialize,
    {
        if args.is_empty() {
            return Ok(0);
        }
        //copy data to trash
        let trash = SysTrash {
            id: Some(ObjectId::new().to_string().into()),
            table_name: Some(table_name.to_string()),
            data: Some(serde_json::to_string(&args).unwrap_or_default()),
            create_date: Some(FastDateTime::now()),
        };
        Ok(SysTrash::insert(pool!(), &trash).await?.rows_affected)
    }
}

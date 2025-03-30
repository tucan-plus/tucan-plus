pub struct Database {
    #[cfg(target_arch = "wasm32")]
    database: indexed_db::Database<std::io::Error>,
    #[cfg(not(target_arch = "wasm32"))]
    database: sqlx::Pool<sqlx::Sqlite>,
}

impl Database {
    pub async fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let factory = indexed_db::Factory::<std::io::Error>::get().unwrap();

            let database = factory
                .open("database", 2, |evt| async move {
                    let db = evt.database();
                    if evt.old_version() == 1 {
                        db.delete_object_store("store")?;
                    }
                    db.build_object_store("store").create()?;
                    Ok(())
                })
                .await
                .unwrap();

            Database { database }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let database = if cfg!(target_os = "android") {
                tokio::fs::create_dir_all("/data/data/de.selfmade4u.tucant/files").await.unwrap();
                sqlx::SqlitePool::connect("sqlite:///data/data/de.selfmade4u.tucant/files/data.db?mode=rwc").await.unwrap()
            } else {
                sqlx::SqlitePool::connect("sqlite://data.db?mode=rwc").await.unwrap()
            };
            let version: u32 = sqlx::query_scalar("PRAGMA user_version").fetch_one(&database).await.unwrap();
            if version != 2 {
                sqlx::query("DROP TABLE IF EXISTS store").execute(&database).await.unwrap();
                sqlx::query("PRAGMA user_version = 2").execute(&database).await.unwrap();
            }
            sqlx::query("CREATE TABLE IF NOT EXISTS store (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)").execute(&database).await.unwrap();
            Self { database }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new_test() -> Self {
        let database = sqlx::SqlitePool::connect("sqlite://:memory:").await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS store (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)").execute(&database).await.unwrap();
        Self { database }
    }

    pub async fn get<V: serde::de::DeserializeOwned>(&self, key: &str) -> Option<V> {
        #[cfg(target_arch = "wasm32")]
        {
            let key = js_sys::wasm_bindgen::JsValue::from(key);
            let result = self
                .database
                .transaction(&["store"])
                .run(|t| async move {
                    let store = t.object_store("store")?;
                    let value = store.get(&key).await.unwrap();

                    Ok(value)
                })
                .await
                .unwrap();
            result.map(|result| serde_wasm_bindgen::from_value(result).unwrap())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            #[derive(sqlx::FromRow)]
            struct Value {
                value: String,
            }

            let result = sqlx::query_as::<_, Value>("SELECT value FROM store WHERE key = ?").bind(key).fetch_optional(&self.database).await.unwrap();
            result.map(|result| serde_json::from_str(&result.value).unwrap())
        }
    }

    pub async fn put<V: serde::ser::Serialize>(&self, key: &str, value: V) {
        #[cfg(target_arch = "wasm32")]
        {
            let key = js_sys::wasm_bindgen::JsValue::from(key);
            let value = serde_wasm_bindgen::to_value(&value).unwrap();
            self.database
                .transaction(&["store"])
                .rw()
                .run(|t| async move {
                    let store = t.object_store("store")?;
                    store.put_kv(&key, &value).await.unwrap();
                    Ok(())
                })
                .await
                .unwrap();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            sqlx::query("INSERT INTO store (key, value) VALUES (?1, ?2) ON CONFLICT (key) DO UPDATE SET value = ?2 WHERE key = ?1").bind(key).bind(serde_json::to_string(&value).unwrap()).execute(&self.database).await.unwrap();
        }
    }

    pub async fn remove_many(&self, keys: Vec<String>) {
        #[cfg(target_arch = "wasm32")]
        {
            use futures_util::StreamExt;
            self.database
                .transaction(&["store"])
                .rw()
                .run(|t| async move {
                    let store = t.object_store("store")?;

                    futures_util::stream::iter(keys)
                        .for_each(async |key| {
                            let key = js_sys::wasm_bindgen::JsValue::from(key);
                            store.delete(&key).await.unwrap();
                        })
                        .await;

                    Ok(())
                })
                .await
                .unwrap();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            sqlx::query("DELETE FROM store WHERE key IN (SELECT value FROM json_each(?))").bind(serde_json::to_string(&keys).unwrap()).execute(&self.database).await.unwrap();
        }
    }
}

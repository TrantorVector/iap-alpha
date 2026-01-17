/// Document repository for company document metadata
use crate::models::Document;
use crate::{DbError, DbResult};
use sqlx::PgPool;
use uuid::Uuid;

/// Document repository
pub struct DocumentRepository {
    pool: PgPool,
}

impl DocumentRepository {
    /// Create a new document repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find documents by company ID with optional type filter
    pub async fn find_by_company_id(
        &self,
        company_id: Uuid,
        document_type: Option<String>,
    ) -> DbResult<Vec<Document>> {
        let mut query = String::from(
            r#"
            SELECT id, company_id, document_type, period_end_date, fiscal_year, fiscal_quarter,
                   title, storage_key, source_url, file_size, mime_type, created_at, updated_at
            FROM documents
            WHERE company_id = $1
            "#,
        );

        let mut param_index = 2;
        if document_type.is_some() {
            query.push_str(&format!(" AND document_type = ${}", param_index));
            param_index += 1;
        }

        query.push_str(" ORDER BY document_type ASC, period_end_date DESC");

        let mut q = sqlx::query_as::<_, Document>(&query).bind(company_id);

        if let Some(doc_type) = document_type {
            q = q.bind(doc_type);
        }

        let documents = q.fetch_all(&self.pool).await.map_err(DbError::from)?;
        Ok(documents)
    }
}

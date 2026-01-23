/// Document repository for company document metadata
use crate::models::Document;
use crate::{DbError, DbResult};
use sqlx::PgPool;
use uuid::Uuid;

/// Document repository
pub struct DocumentRepository {
    pool: PgPool,
}

/// Parameters for creating a document
pub struct CreateDocumentParams {
    pub company_id: Uuid,
    pub document_type: String,
    pub period_end_date: Option<chrono::NaiveDate>,

    pub title: String,
    pub storage_key: String,
    pub source_url: Option<String>,
    pub file_size: i64,
    pub mime_type: String,
}

impl DocumentRepository {
    /// Create a new document repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find document by ID
    pub async fn find_by_id(&self, id: Uuid) -> DbResult<Option<Document>> {
        let document = sqlx::query_as::<_, Document>(
            r#"
            SELECT id, company_id, document_type, period_end_date,
                   title, storage_key, source_url, file_size, mime_type, created_at, updated_at
            FROM documents
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(document)
    }

    /// Find documents by company ID with optional type filter
    pub async fn find_by_company_id(
        &self,
        company_id: Uuid,
        document_type: Option<String>,
    ) -> DbResult<Vec<Document>> {
        let mut query = String::from(
            r#"
            SELECT id, company_id, document_type, period_end_date,
                   title, storage_key, source_url, file_size, mime_type, created_at, updated_at
            FROM documents
            WHERE company_id = $1
            "#,
        );

        let param_index = 2;
        if document_type.is_some() {
            query.push_str(&format!(" AND document_type = ${}", param_index));
        }

        query.push_str(" ORDER BY document_type ASC, period_end_date DESC");

        let mut q = sqlx::query_as::<_, Document>(&query).bind(company_id);

        if let Some(doc_type) = document_type {
            q = q.bind(doc_type);
        }

        let documents = q.fetch_all(&self.pool).await.map_err(DbError::from)?;
        Ok(documents)
    }

    /// Create a new document
    pub async fn create(&self, params: CreateDocumentParams) -> DbResult<Document> {
        let document = sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (
                id, company_id, document_type, period_end_date,
                title, storage_key, source_url, file_size, mime_type, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            RETURNING id, company_id, document_type, period_end_date,
                      title, storage_key, source_url, file_size, mime_type, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(params.company_id)
        .bind(params.document_type)
        .bind(params.period_end_date)
        .bind(params.title)
        .bind(params.storage_key)
        .bind(params.source_url)
        .bind(params.file_size)
        .bind(params.mime_type)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(document)
    }
}

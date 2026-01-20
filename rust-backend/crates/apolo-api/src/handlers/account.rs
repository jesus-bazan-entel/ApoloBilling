//! Account handlers
//!
//! HTTP handlers for account management endpoints.

use crate::dto::account::{
    AccountCreateRequest, AccountFilterParams, AccountResponse, AccountUpdateRequest, TopupRequest,
    TopupResponse,
};
use crate::dto::{ApiResponse, PaginationParams};
use actix_web::{web, HttpResponse};
use apolo_auth::AuthenticatedUser;
use apolo_core::models::{AccountStatus, AccountType};
use apolo_core::traits::{AccountRepository, Repository};
use apolo_core::AppError;
use apolo_db::PgAccountRepository;
use chrono::Utc;
use sqlx::PgPool;
use tracing::{debug, info, instrument, warn};
use validator::Validate;

/// List accounts with pagination and filters
///
/// GET /api/v1/accounts
#[instrument(skip(pool, _user))]
pub async fn list_accounts(
    pool: web::Data<PgPool>,
    query: web::Query<PaginationParams>,
    filters: web::Query<AccountFilterParams>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    query.validate().map_err(|e| {
        warn!("Pagination validation failed: {}", e);
        AppError::Validation(e.to_string())
    })?;

    debug!(
        page = query.page,
        per_page = query.per_page,
        "Listing accounts"
    );

    let repo = PgAccountRepository::new(pool.get_ref().clone());

    // Pass filter parameters as string references
    let status = filters.status.as_deref();
    let account_type = filters.account_type.as_deref();

    // Get accounts with filters
    let (accounts, total) = repo
        .list_filtered(status, account_type, query.limit(), query.offset())
        .await?;

    let response_data: Vec<AccountResponse> = accounts.into_iter().map(|a| a.into()).collect();

    Ok(HttpResponse::Ok().json(query.paginate(response_data, total)))
}

/// Create a new account
///
/// POST /api/v1/accounts
#[instrument(skip(pool, _user, req))]
pub async fn create_account(
    pool: web::Data<PgPool>,
    _user: AuthenticatedUser,
    req: web::Json<AccountCreateRequest>,
) -> Result<HttpResponse, AppError> {
    req.validate().map_err(|e| {
        warn!("Account creation validation failed: {}", e);
        AppError::Validation(e.to_string())
    })?;

    debug!(account_number = %req.account_number, "Creating account");

    let repo = PgAccountRepository::new(pool.get_ref().clone());

    // Check if account number already exists
    if let Some(_existing) = repo.find_by_number(&req.account_number).await? {
        warn!(
            account_number = %req.account_number,
            "Account creation failed: duplicate account number"
        );
        return Err(AppError::AlreadyExists(format!(
            "Account {} already exists",
            req.account_number
        )));
    }

    // Create account
    let account = req.to_account();
    let created = repo.create(&account).await?;

    info!(
        id = created.id,
        account_number = %created.account_number,
        "Account created successfully"
    );

    let response = AccountResponse::from(created);
    Ok(HttpResponse::Created().json(ApiResponse::with_message(
        response,
        "Account created successfully",
    )))
}

/// Get a single account by ID
///
/// GET /api/v1/accounts/{id}
#[instrument(skip(pool, _user))]
pub async fn get_account(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let account_id = path.into_inner();
    debug!(id = account_id, "Getting account");

    let repo = PgAccountRepository::new(pool.get_ref().clone());
    let account = repo
        .find_by_id(account_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Account {} not found", account_id)))?;

    let response = AccountResponse::from(account);
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// Update an account
///
/// PUT /api/v1/accounts/{id}
#[instrument(skip(pool, _user, req))]
pub async fn update_account(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    _user: AuthenticatedUser,
    req: web::Json<AccountUpdateRequest>,
) -> Result<HttpResponse, AppError> {
    req.validate().map_err(|e| {
        warn!("Account update validation failed: {}", e);
        AppError::Validation(e.to_string())
    })?;

    let account_id = path.into_inner();
    debug!(id = account_id, "Updating account");

    let repo = PgAccountRepository::new(pool.get_ref().clone());

    // Get existing account
    let mut account = repo
        .find_by_id(account_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Account {} not found", account_id)))?;

    // Apply updates
    if let Some(phone) = &req.customer_phone {
        account.customer_phone = Some(phone.clone());
    }

    if let Some(status_str) = &req.status {
        if let Some(status) = AccountStatus::from_str(status_str) {
            account.status = status;
        } else {
            return Err(AppError::Validation(format!(
                "Invalid status: {}",
                status_str
            )));
        }
    }

    if let Some(credit_limit) = req.credit_limit {
        account.credit_limit = credit_limit;
    }

    if let Some(max_calls) = req.max_concurrent_calls {
        account.max_concurrent_calls = max_calls;
    }

    account.updated_at = Utc::now();

    // Save updates
    let updated = repo.update(&account).await?;

    info!(
        id = updated.id,
        account_number = %updated.account_number,
        "Account updated successfully"
    );

    let response = AccountResponse::from(updated);
    Ok(HttpResponse::Ok().json(ApiResponse::with_message(
        response,
        "Account updated successfully",
    )))
}

/// Top up account balance
///
/// POST /api/v1/accounts/{id}/topup
#[instrument(skip(pool, _user, req))]
pub async fn topup_account(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    _user: AuthenticatedUser,
    req: web::Json<TopupRequest>,
) -> Result<HttpResponse, AppError> {
    req.validate().map_err(|e| {
        warn!("Topup validation failed: {}", e);
        AppError::Validation(e.to_string())
    })?;

    let account_id = path.into_inner();
    debug!(id = account_id, amount = %req.amount, "Processing topup");

    let repo = PgAccountRepository::new(pool.get_ref().clone());

    // Get existing account
    let account = repo
        .find_by_id(account_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Account {} not found", account_id)))?;

    let previous_balance = account.balance;

    // Update balance
    let new_balance = repo.update_balance(account_id, req.amount).await?;

    info!(
        id = account_id,
        amount = %req.amount,
        previous_balance = %previous_balance,
        new_balance = %new_balance,
        "Topup successful"
    );

    let response = TopupResponse::new(previous_balance, req.amount, new_balance);
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// Delete an account
///
/// DELETE /api/v1/accounts/{id}
#[instrument(skip(pool, admin))]
pub async fn delete_account(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    admin: apolo_auth::AdminUser,
) -> Result<HttpResponse, AppError> {
    let account_id = path.into_inner();
    debug!(
        id = account_id,
        admin = %admin.username,
        "Deleting account"
    );

    let repo = PgAccountRepository::new(pool.get_ref().clone());

    // Verify account exists
    let _account = repo
        .find_by_id(account_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Account {} not found", account_id)))?;

    // Delete account
    let deleted = repo.delete(account_id).await?;

    if deleted {
        info!(
            id = account_id,
            admin = %admin.username,
            "Account deleted successfully"
        );
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(AppError::Internal("Failed to delete account".to_string()))
    }
}

/// Configure account routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/accounts")
            .route("", web::get().to(list_accounts))
            .route("", web::post().to(create_account))
            .route("/{id}", web::get().to(get_account))
            .route("/{id}", web::put().to(update_account))
            .route("/{id}", web::delete().to(delete_account))
            .route("/{id}/topup", web::post().to(topup_account)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_topup_request_validation() {
        let valid_req = TopupRequest {
            amount: dec!(50.00),
            reason: Some("Monthly recharge".to_string()),
        };
        assert!(valid_req.validate().is_ok());

        // Note: validator's range doesn't work well with Decimal,
        // so we rely on business logic validation
    }
}

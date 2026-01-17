# Investment Analysis Platform (iap-alpha)

**I**nvestment **A**nalysis **P**latform - **Alpha**

A comprehensive platform for fundamental stock analysis, combining automated data collection, multi-period financial metrics analysis, and an intelligent stock screener.

## 🚀 Quick Start

### Prerequisites

- Docker and Docker Compose
- Git

### Running with Docker (Recommended)

```bash
# Clone the repository
git clone git@github.com:TrantorVector/iap-alpha.git
cd iap-alpha

# Copy environment variables
cp .env.example .env
# Edit .env and add your Alpha Vantage API key

# Start all services
docker-compose up -d

# Access the application
# Frontend: http://localhost:3000
# API: http://localhost:8000
```

## 📚 Documentation

Comprehensive documentation is available in the [`docs/`](./docs/) folder:

- **Architecture Design**: [`docs/architecture-design-v3.md`](./docs/architecture-design-v3.md)
- **Build Plan**: [`docs/build-plan-v3/`](./docs/build-plan-v3/)
- **Database Design**: [`docs/database-design-v1.md`](./docs/database-design-v1.md)
- **PRD**: [`docs/prd-v1-4.md`](./docs/prd-v1-4.md)

## 🛠️ Technology Stack

### Backend
- **Language**: Rust
- **Framework**: Axum (async web framework)
- **Database**: PostgreSQL 16
- **ORM**: SQLx (compile-time verified queries)
- **API**: RESTful with JWT authentication

### Frontend
- **Framework**: React with TypeScript
- **Build Tool**: Vite
- **State Management**: Zustand
- **Styling**: Tailwind CSS
- **UI Components**: Radix UI / shadcn/ui

### Infrastructure
- **Container Orchestration**: Docker & Docker Compose (development)
- **Cloud Platform**: AWS ECS Fargate (production)
- **IaC**: Pulumi with TypeScript
- **Object Storage**: MinIO (dev) / AWS S3 (prod)
- **CI/CD**: GitHub Actions

### Data Provider
- **Primary**: Alpha Vantage API (free tier)
- **Mock Provider**: For testing and development

## 🏗️ Project Structure

```
iap-alpha/
├── backend/          # Rust backend services
│   ├── api/         # REST API layer
│   ├── core/        # Business logic & domain
│   ├── db/          # Database models & migrations
│   ├── worker/      # Background jobs
│   └── providers/   # External data providers
├── frontend/        # React TypeScript frontend
├── infra/           # Infrastructure as Code (Pulumi)
├── tests/           # Integration & E2E tests
└── docs/            # Documentation
```

## 🔑 Features

### 📊 Analyzer Module
- Comprehensive fundamental analysis of individual stocks
- Multi-period financial metrics (3Y, 5Y, 10Y, LTM)
- Interactive visualizations and trend analysis
- Export analysis reports in multiple formats

### 🔍 Screener Module
- Filter stocks based on custom criteria
- Save and reuse screening strategies
- Real-time screening results
- Bulk analysis capabilities

### 📈 Results Tracker
- Track screening results over time
- Monitor performance of screened stocks
- Historical comparison and backtesting

## 📝 Development Status

This is an **alpha version** of the platform, currently under active development. The project is being built incrementally following a structured build plan.

Current Phase: **Repository Initialization & Setup**

## 📄 License

[License TBD]

## 🤝 Contributing

This is a personal project. Contributions, issues, and feature requests are welcome once the alpha version is stable.

---

**Built with ❤️ and AI assistance (Antigravity)**

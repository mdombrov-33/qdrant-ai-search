# ===============================================================================
# 🛠️  Makefile for Qdrant AI Search Platform
# ===============================================================================
#
# 🧩 Project Structure:
# - backend/              : FastAPI service (Python)
# - rust_accelerator/     : Rust microservice (high-performance reranker)
# - qdrant                : Vector DB (via Docker image)
#
# 📦 Versioning System:
# - backend/.version              → contains tag like `v1`, `v2`, etc.
# - rust_accelerator/.version     → same format for Rust service
#
# These version tags are used to:
# - build Docker images (e.g. backend:v2)
# - tag Helm deployments
# - populate Docker Compose `.env` for local dev
#
# ===============================================================================
# 🧠 DAILY WORKFLOW SUMMARY
# ===============================================================================
#
# ▶ Local Dev via Docker Compose:
#   1. make sync-compose-env          # Sync .env with current image versions
#   2. docker-compose -f docker-compose.dev.yml up --build
#
# ▶ Cluster Dev via Minikube + Helm:
#   1. make bump-backend-version      # Optional: If backend changed
#   2. make deploy-backend            # Build, tag, load, helm upgrade backend
#   3. make bump-rust-version         # Optional: If rust changed
#   4. make deploy-rust               # Build, tag, load, helm upgrade rust
#   5. make deploy-qdrant             # Only needed once or on config change
#   6. make restart-*                 # Restart deployments without rebuilding
#
# ▶ Quality & Maintenance:
#   - make format-all lint-all        # Clean + check code (Python & Rust)
#   - make deploy-all                 # Deploy all services (backend, rust, qdrant)
#
# ===============================================================================

NAMESPACE := qdrant-ai

# Read current image tags from version files
BACKEND_IMAGE_TAG := $(shell cat backend/.version)
RUST_IMAGE_TAG := $(shell cat rust_accelerator/.version)

BACKEND_IMAGE := backend:$(BACKEND_IMAGE_TAG)
RUST_IMAGE := rust-accelerator:$(RUST_IMAGE_TAG)

# ===============================================================================
# 🎨 Code Quality
# ===============================================================================

format-python:
	poetry run --directory backend black .

lint-python:
	poetry run --directory backend ruff check .

format-rust:
	cargo fmt --manifest-path rust_accelerator/Cargo.toml

lint-rust:
	cargo clippy --manifest-path rust_accelerator/Cargo.toml --all-targets --all-features -- -D warnings

format-all: format-python format-rust
lint-all: lint-python lint-rust

# ===============================================================================
# 🔁 Version Management
# ===============================================================================

bump-backend-version:
	@echo "🔖 Bumping backend version..."
	@current=$$(cat backend/.version); \
	 minor=$${current#v}; \
	 new_minor=$$(($${minor}+1)); \
	 new_version="v$${new_minor}"; \
	 echo $${new_version} > backend/.version; \
	 echo "✅ New backend version: $${new_version}"

bump-rust-version:
	@echo "🔖 Bumping rust version..."
	@current=$$(cat rust_accelerator/.version); \
	 minor=$${current#v}; \
	 new_minor=$$(($${minor}+1)); \
	 new_version="v$${new_minor}"; \
	 echo $${new_version} > rust_accelerator/.version; \
	 echo "✅ New rust version: $${new_version}"

# ===============================================================================
# 🐳 Docker Image Builds (Minikube/K8s)
# ===============================================================================

build-backend:
	@echo "🐍 Building backend image $(BACKEND_IMAGE)..."
	docker build -t $(BACKEND_IMAGE) -f backend/Dockerfile ./backend
	minikube image load $(BACKEND_IMAGE)

build-rust:
	@echo "🦀 Building rust image $(RUST_IMAGE)..."
	docker build -t $(RUST_IMAGE) -f rust_accelerator/Dockerfile ./rust_accelerator
	minikube image load $(RUST_IMAGE)

# ===============================================================================
# ☸️ Helm + Kubernetes Deployment
# ===============================================================================

deploy-backend: build-backend
	@echo "🚀 Deploying backend via Helm with image $(BACKEND_IMAGE_TAG)..."
	helm upgrade --install backend ./helm/backend -n $(NAMESPACE) \
	  --set image.tag=$(BACKEND_IMAGE_TAG) \
	  --set image.pullPolicy=IfNotPresent

deploy-rust: build-rust
	@echo "🚀 Deploying rust-accelerator via Helm with image $(RUST_IMAGE_TAG)..."
	helm upgrade --install rust-accelerator ./helm/rust-accelerator -n $(NAMESPACE) \
	  --set image.tag=$(RUST_IMAGE_TAG) \
	  --set image.pullPolicy=IfNotPresent

deploy-qdrant:
	@echo "🚀 Deploying Qdrant via Helm..."
	helm upgrade --install qdrant ./helm/qdrant -n $(NAMESPACE)

# ===============================================================================
# 🐳 Docker Compose: Sync .env from version files
# ===============================================================================

sync-compose-env:
	@echo "🔄 Writing backend + rust versions into .env for Docker Compose..."
	@echo "BACKEND_IMAGE_TAG=$(BACKEND_IMAGE_TAG)" > .env
	@echo "RUST_IMAGE_TAG=$(RUST_IMAGE_TAG)" >> .env
	@echo "✅ .env written: backend=$(BACKEND_IMAGE_TAG), rust=$(RUST_IMAGE_TAG)"

# ===============================================================================
# 🔁 Restart Deployed K8s Services (no rebuild)
# ===============================================================================

restart-backend:
	kubectl rollout restart deployment/backend -n $(NAMESPACE)

restart-rust:
	kubectl rollout restart deployment/rust-accelerator -n $(NAMESPACE)

restart-qdrant:
	kubectl rollout restart deployment/qdrant -n $(NAMESPACE)

# ===============================================================================
# 🧪 Debug / Misc Utilities
# ===============================================================================

deploy-all: deploy-backend deploy-rust deploy-qdrant

status:
	kubectl get pods -n $(NAMESPACE)

# Usage: make logs SERVICE=backend
logs:
	kubectl logs -n $(NAMESPACE) deployment/$(SERVICE) -f

# Usage: make port SERVICE=backend PORT=8000
port:
	kubectl port-forward -n $(NAMESPACE) deployment/$(SERVICE) $(PORT):$(PORT)

.PHONY: format-python lint-python format-rust lint-rust format-all lint-all \
        build-backend build-rust deploy-backend deploy-rust deploy-qdrant \
        restart-backend restart-rust restart-qdrant deploy-all status logs port \
        bump-backend-version bump-rust-version sync-compose-env

# ===============================================================================
# Makefile for Qdrant AI Search - 3 Services: Backend, Rust, Qdrant
# =============================================================================== 
#
# WHAT WE HAVE:
# - backend/ (Python FastAPI) 
# - rust_accelerator/ (Rust microservice)
# - qdrant (official vector DB)
#
# DAILY WORKFLOW:
# 1. make format-all lint-all          (clean code)
# 2. make bump-backend-version        (increment backend version tag)
# 3. make deploy-backend              (build + deploy backend with new version)
# 4. make bump-rust-version           (increment rust version tag)
# 5. make deploy-rust                 (build + deploy rust with new version)
# 6. make restart-*                   (restart pods if stuck)
#
# CRITICAL LESSON LEARNED: imagePullPolicy=Never prevents K8s image caching hell
#
# IMPORTANT NOTE ABOUT IMAGE TAGGING:
# To avoid Kubernetes running stale cached images when using 'imagePullPolicy=Never',
# we use explicit versioned image tags instead of 'latest'. This ensures K8s loads
# the correct new image after each rebuild and deploy.
#
# Workflow:
# 1. Increment tags on each rebuild using make bump-backend-version / bump-rust-version
# 2. Build and load image with that tag into minikube
# 3. Deploy with Helm setting image.tag and image.pullPolicy=IfNotPresent
# 4. Restart deployment to pick up new image (if needed)
#
# This eliminates issues with stale 'latest' tags being cached by Kubernetes.
#
# To initialize versioning, create files:
#   backend/.version        # contains "v1" or current backend version tag
#   rust_accelerator/.version # contains "v1" or current rust version tag
#
# ===============================================================================

NAMESPACE := qdrant-ai

# Read current versions from version files
BACKEND_IMAGE_TAG := $(shell cat backend/.version)
BACKEND_IMAGE := backend:$(BACKEND_IMAGE_TAG)

RUST_IMAGE_TAG := $(shell cat rust_accelerator/.version)
RUST_IMAGE := rust-accelerator:$(RUST_IMAGE_TAG)

# ===============================================================================
# Code formatting and linting
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
# Version bumping commands (increments v1 → v2, etc.)
# ===============================================================================

bump-backend-version:
	@echo "Bumping backend version..."
	@current=$$(cat backend/.version); \
	major=$${current%[a-z]*}; \
	minor=$${current#v}; \
	new_minor=$$(($${minor}+1)); \
	new_version="v$${new_minor}"; \
	echo $${new_version} > backend/.version; \
	echo "New backend version: $${new_version}"

bump-rust-version:
	@echo "Bumping rust version..."
	@current=$$(cat rust_accelerator/.version); \
	major=$${current%[a-z]*}; \
	minor=$${current#v}; \
	new_minor=$$(($${minor}+1)); \
	new_version="v$${new_minor}"; \
	echo $${new_version} > rust_accelerator/.version; \
	echo "New rust version: $${new_version}"

# ===============================================================================
# Docker image building (builds + loads into minikube)
# ===============================================================================

build-backend:
	@echo "🐍 Building backend image with tag $(BACKEND_IMAGE_TAG)..."
	docker build -t $(BACKEND_IMAGE) -f backend/Dockerfile ./backend
	minikube image load $(BACKEND_IMAGE)

build-rust:
	@echo "🦀 Building rust image with tag $(RUST_IMAGE_TAG)..."
	docker build -t $(RUST_IMAGE) -f rust_accelerator/Dockerfile ./rust_accelerator
	minikube image load $(RUST_IMAGE)

# ===============================================================================
# Kubernetes deployment (build + helm deploy)
# ===============================================================================

deploy-backend: build-backend
	@echo "🚀 Deploying backend with image tag $(BACKEND_IMAGE_TAG)..."
	helm upgrade --install backend ./helm/backend -n $(NAMESPACE) \
	  --set image.tag=$(BACKEND_IMAGE_TAG) \
	  --set image.pullPolicy=IfNotPresent

deploy-rust: build-rust
	@echo "🚀 Deploying rust accelerator with image tag $(RUST_IMAGE_TAG)..."
	helm upgrade --install rust-accelerator ./helm/rust-accelerator -n $(NAMESPACE) \
	  --set image.tag=$(RUST_IMAGE_TAG) \
	  --set image.pullPolicy=IfNotPresent

deploy-qdrant:
	@echo "🚀 Deploying qdrant..."
	helm upgrade --install qdrant ./helm/qdrant -n $(NAMESPACE)

# ===============================================================================
# Restart services (without rebuilding)
# ===============================================================================

restart-backend:
	kubectl rollout restart deployment/backend -n $(NAMESPACE)

restart-rust:
	kubectl rollout restart deployment/rust-accelerator -n $(NAMESPACE)

restart-qdrant:
	kubectl rollout restart deployment/qdrant -n $(NAMESPACE)

# ===============================================================================
# Convenience targets
# ===============================================================================

# Deploy everything from scratch
deploy-all: deploy-backend deploy-rust deploy-qdrant

# Check what's running
status:
	kubectl get pods -n $(NAMESPACE)

# Follow logs (usage: make logs SERVICE=backend)
logs:
	kubectl logs -n $(NAMESPACE) deployment/$(SERVICE) -f

# Port forward for testing (usage: make port SERVICE=backend PORT=8000)
port:
	kubectl port-forward -n $(NAMESPACE) deployment/$(SERVICE) $(PORT):$(PORT)

.PHONY: format-python lint-python format-rust lint-rust format-all lint-all \
        build-backend build-rust deploy-backend deploy-rust deploy-qdrant \
        restart-backend restart-rust restart-qdrant deploy-all status logs port \
        bump-backend-version bump-rust-version

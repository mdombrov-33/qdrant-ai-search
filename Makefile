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
# 1. make format-all lint-all  (clean code)
# 2. make deploy-backend       (after changing Python)
# 3. make deploy-rust          (after changing Rust)
# 4. make restart-*            (when pods are stuck)
#
# CRITICAL LESSON LEARNED: imagePullPolicy=Never prevents K8s image caching hell
#
# ===============================================================================

NAMESPACE := qdrant-ai
BACKEND_IMAGE := backend:latest
RUST_IMAGE := rust-accelerator:latest

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
# Docker image building (builds + loads into minikube)
# ===============================================================================

build-backend:
	@echo "🐍 Building backend image..."
	docker build -t $(BACKEND_IMAGE) -f backend/Dockerfile ./backend
	minikube image load $(BACKEND_IMAGE)

build-rust:
	@echo "🦀 Building rust image..."
	docker build -t $(RUST_IMAGE) -f rust_accelerator/Dockerfile ./rust_accelerator
	minikube image load $(RUST_IMAGE)

# ===============================================================================
# Kubernetes deployment (build + helm deploy)
# ===============================================================================

deploy-backend: build-backend
	@echo "🚀 Deploying backend..."
	helm upgrade --install backend ./helm/backend -n $(NAMESPACE) --set image.pullPolicy=Never

deploy-rust: build-rust
	@echo "🚀 Deploying rust accelerator..."
	helm upgrade --install rust-accelerator ./helm/rust-accelerator -n $(NAMESPACE) --set image.pullPolicy=Never

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
        restart-backend restart-rust restart-qdrant deploy-all status logs port
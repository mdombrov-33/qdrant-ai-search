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
# - tag Helm deployments to Kubernetes
# - populate Docker Compose `.env` for local testing of specific versions
#
# ===============================================================================
# 🧠 DAILY WORKFLOWS - Choose Your Adventure
# ===============================================================================
#
# 🔨 SCENARIO 1: Active Development
# "I'm coding and want to see changes immediately"
# 
#   docker-compose -f docker-compose.dev.yml up --build
# 
# → Uses dev.yml which builds fresh from source code
# → No versioning, no Makefile needed
# → Perfect for rapid iteration
#
# ─────────────────────────────────────────────────────────────────────────────
#
# 🧪 SCENARIO 2: Pre-Deployment Testing  
# "I want to test the exact same images that will go to Kubernetes"
#
#   1. make bump-backend-version      # Creates new version (v1 → v2)
#   2. make build-backend-for-compose # Builds backend:v2 locally
#   3. make bump-rust-version         # Creates new version  
#   4. make build-rust-for-compose    # Builds rust-accelerator:v2 locally
#   5. make sync-compose-env          # Writes versions to .env
#   6. docker-compose -f docker-compose.prod.yml up
#
# → Uses prod.yml which pulls tagged images (backend:v2, rust:v2)
# → Tests the exact same images that Kubernetes will run
# → Catches integration issues before deployment
#
# ─────────────────────────────────────────────────────────────────────────────
#
# ☸️ SCENARIO 3: Deploy to Kubernetes
# "Ready to deploy to my cluster"
#
#   make deploy-all                   # Deploys all services with current versions
#
# OR step-by-step:
#   1. make deploy-backend            # Build + load into Minikube + Helm deploy
#   2. make deploy-rust               # Build + load into Minikube + Helm deploy  
#   3. make deploy-qdrant             # Deploy Qdrant (only needed once)
#
# → Builds images, loads them into Minikube, deploys via Helm
# → Uses the versions from .version files
#
# ─────────────────────────────────────────────────────────────────────────────
#
# 🔁 SCENARIO 4: Restart Services (No Rebuild)
# "Something's stuck, restart without rebuilding"
#
#   make restart-backend              # Just restart the backend pods
#   make restart-rust                 # Just restart the rust pods
#   make status                       # Check what's running
#
# ===============================================================================

NAMESPACE := qdrant-ai

# Read current image tags from version files
BACKEND_IMAGE_TAG := $(shell cat backend/.version)
RUST_IMAGE_TAG := $(shell cat rust_accelerator/.version)

BACKEND_IMAGE := backend:$(BACKEND_IMAGE_TAG)
RUST_IMAGE := rust-accelerator:$(RUST_IMAGE_TAG)

# ===============================================================================
# 🎨 Code Quality (Run Before Committing)
# ===============================================================================

format-python:
	@echo "🐍 Formatting Python code with Black..."
	poetry run --directory backend black .

lint-python:
	@echo "🐍 Linting Python code with Ruff..."
	poetry run --directory backend ruff check .

format-rust:
	@echo "🦀 Formatting Rust code..."
	cargo fmt --manifest-path rust_accelerator/Cargo.toml

lint-rust:
	@echo "🦀 Linting Rust code with Clippy..."
	cargo clippy --manifest-path rust_accelerator/Cargo.toml --all-targets --all-features -- -D warnings

format-all: format-python format-rust
	@echo "✅ All code formatted!"

lint-all: lint-python lint-rust
	@echo "✅ All code linted!"

# ===============================================================================
# 🔁 Version Management (Bump Before Testing/Deploying)
# ===============================================================================

bump-backend-version:
	@echo "🔖 Bumping backend version..."
	@current=$$(cat backend/.version); \
	 minor=$${current#v}; \
	 new_minor=$$(($${minor}+1)); \
	 new_version="v$${new_minor}"; \
	 echo $${new_version} > backend/.version; \
	 echo "✅ Backend version: $${current} → $${new_version}"

bump-rust-version:
	@echo "🔖 Bumping rust version..."
	@current=$$(cat rust_accelerator/.version); \
	 minor=$${current#v}; \
	 new_minor=$$(($${minor}+1)); \
	 new_version="v$${new_minor}"; \
	 echo $${new_version} > rust_accelerator/.version; \
	 echo "✅ Rust version: $${current} → $${new_version}"

# ===============================================================================
# 🐳 Docker Image Builds
# ===============================================================================

# Build images for Kubernetes (loads into Minikube)
build-backend:
	@echo "🐍 Building backend image $(BACKEND_IMAGE) for Kubernetes..."
	docker build -t $(BACKEND_IMAGE) -f backend/Dockerfile ./backend
	minikube image load $(BACKEND_IMAGE)
	@echo "✅ Backend image loaded into Minikube"

build-rust:
	@echo "🦀 Building rust image $(RUST_IMAGE) for Kubernetes..."
	docker build -t $(RUST_IMAGE) -f rust_accelerator/Dockerfile ./rust_accelerator
	minikube image load $(RUST_IMAGE)
	@echo "✅ Rust image loaded into Minikube"

# Build images for local Docker Compose testing (no Minikube load)
build-backend-for-compose:
	@echo "🐍 Building backend image $(BACKEND_IMAGE) for local testing..."
	docker build -t $(BACKEND_IMAGE) -f backend/Dockerfile ./backend
	@echo "✅ Backend image ready for docker-compose.prod.yml"

build-rust-for-compose:
	@echo "🦀 Building rust image $(RUST_IMAGE) for local testing..."
	docker build -t $(RUST_IMAGE) -f rust_accelerator/Dockerfile ./rust_accelerator
	@echo "✅ Rust image ready for docker-compose.prod.yml"

# ===============================================================================
# ☸️ Helm + Kubernetes Deployment
# ===============================================================================

deploy-backend: build-backend
	@echo "🚀 Deploying backend via Helm with image $(BACKEND_IMAGE_TAG)..."
	@if [ -n "$(OPENAI_API_KEY)" ]; then \
		echo "Using OPENAI_API_KEY from environment..."; \
		helm upgrade --install backend ./helm/backend -n $(NAMESPACE) \
		  --set image.tag=$(BACKEND_IMAGE_TAG) \
		  --set secrets.useKubernetesSecret=false \
		  --set env.OPENAI_API_KEY="$(OPENAI_API_KEY)"; \
	else \
		echo "Using Kubernetes secret for OPENAI_API_KEY..."; \
		helm upgrade --install backend ./helm/backend -n $(NAMESPACE) \
		  --set image.tag=$(BACKEND_IMAGE_TAG); \
	fi
	@echo "✅ Backend deployed to Kubernetes"

deploy-rust: build-rust
	@echo "🚀 Deploying rust-accelerator via Helm with image $(RUST_IMAGE_TAG)..."
	helm upgrade --install rust-accelerator ./helm/rust-accelerator -n $(NAMESPACE) \
	  --set image.tag=$(RUST_IMAGE_TAG) \
	  --set image.pullPolicy=IfNotPresent
	@echo "✅ Rust accelerator deployed to Kubernetes"

deploy-qdrant:
	@echo "🚀 Deploying Qdrant via Helm..."
	helm upgrade --install qdrant ./helm/qdrant -n $(NAMESPACE)
	@echo "✅ Qdrant deployed to Kubernetes"

# ===============================================================================
# 🐳 Docker Compose Support
# ===============================================================================

sync-compose-env:
	@echo "🔄 Syncing .env file with current image versions..."
	@echo "# Auto-generated by 'make sync-compose-env'" > .env
	@echo "# Used by docker-compose.prod.yml to pull specific image versions" >> .env
	@echo "BACKEND_IMAGE_TAG=$(BACKEND_IMAGE_TAG)" >> .env
	@echo "RUST_IMAGE_TAG=$(RUST_IMAGE_TAG)" >> .env
	@echo "✅ .env written: backend=$(BACKEND_IMAGE_TAG), rust=$(RUST_IMAGE_TAG)"
	@echo "💡 Now run: docker-compose -f docker-compose.prod.yml up"

# ===============================================================================
# 🔁 Kubernetes Service Management (No Rebuild)
# ===============================================================================

restart-backend:
	@echo "🔄 Restarting backend deployment..."
	kubectl rollout restart deployment/backend -n $(NAMESPACE)

restart-rust:
	@echo "🔄 Restarting rust-accelerator deployment..."
	kubectl rollout restart deployment/rust-accelerator -n $(NAMESPACE)

restart-qdrant:
	@echo "🔄 Restarting Qdrant deployment..."
	kubectl rollout restart deployment/qdrant -n $(NAMESPACE)

# ===============================================================================
# 🧪 Debug & Monitoring Utilities
# ===============================================================================

deploy-all: deploy-backend deploy-rust deploy-qdrant
	@echo "🎉 All services deployed to Kubernetes!"

status:
	@echo "📊 Kubernetes pod status:"
	kubectl get pods -n $(NAMESPACE) -o wide

# Usage: make logs SERVICE=backend
logs:
	@echo "📋 Tailing logs for $(SERVICE)..."
	kubectl logs -n $(NAMESPACE) deployment/$(SERVICE) -f

# Usage: make port SERVICE=backend PORT=8000
port:
	@echo "🔌 Port forwarding $(SERVICE) on port $(PORT)..."
	kubectl port-forward -n $(NAMESPACE) deployment/$(SERVICE) $(PORT):$(PORT)

# Show helpful command examples
help:
	@echo "🚀 Qdrant AI Search Platform - Quick Commands"
	@echo ""
	@echo "📈 Development Workflow:"
	@echo "  make format-all lint-all                    # Clean up code"
	@echo "  docker-compose -f docker-compose.dev.yml up --build  # Live development"
	@echo ""
	@echo "🧪 Testing Workflow:"
	@echo "  make bump-backend-version build-backend-for-compose"
	@echo "  make bump-rust-version build-rust-for-compose"
	@echo "  make sync-compose-env"
	@echo "  docker-compose -f docker-compose.prod.yml up         # Test specific versions"
	@echo ""
	@echo "☸️ Kubernetes Deployment:"
	@echo "  make deploy-all                             # Deploy everything"
	@echo "  make status                                 # Check pod status"
	@echo "  make logs SERVICE=backend                   # View logs"
	@echo "  make port SERVICE=backend PORT=8000         # Port forward"
	@echo ""

.PHONY: format-python lint-python format-rust lint-rust format-all lint-all \
        build-backend build-rust build-backend-for-compose build-rust-for-compose \
        deploy-backend deploy-rust deploy-qdrant restart-backend restart-rust restart-qdrant \
        deploy-all status logs port bump-backend-version bump-rust-version sync-compose-env help
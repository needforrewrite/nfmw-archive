FROM archlinux:latest AS builder

SHELL ["/bin/bash", "-c"]

# Keep package lists minimal and install build deps
RUN pacman -Sy --noconfirm --needed \
	archlinux-keyring \
 && pacman-key --init \
 && pacman-key --populate archlinux \
 && pacman -Syu --noconfirm --needed \
	base-devel \
	git \
	curl \
	ca-certificates \
	cmake \
	pkgconf \
	openssl \
	zlib \
	icu \
	libunwind \
	python \
 && pacman -Scc --noconfirm

# Install .NET SDK (use dotnet-install script and request channel 10)
ENV DOTNET_INSTALL_DIR=/usr/share/dotnet
RUN curl -sSL https://dot.net/v1/dotnet-install.sh -o /tmp/dotnet-install.sh \
 && bash /tmp/dotnet-install.sh --channel 10.0 --install-dir ${DOTNET_INSTALL_DIR} --no-path \
 && ln -s ${DOTNET_INSTALL_DIR}/dotnet /usr/bin/dotnet \
 && rm -f /tmp/dotnet-install.sh

# Install Rust toolchain via rustup
ENV RUSTUP_HOME=/root/.rustup \
	CARGO_HOME=/root/.cargo \
	PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:${DOTNET_INSTALL_DIR}
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
 && rustup default nightly

# Create app directory and copy sources
WORKDIR /app
COPY . /app

# Build the project (build.rs may invoke the dotnet generator)
RUN cargo build --release

FROM archlinux:latest AS runtime
SHELL ["/bin/bash", "-c"]

# Runtime deps required by the binary and .NET-generated artifacts
RUN pacman -Sy --noconfirm --needed \
	ca-certificates \
	openssl \
	zlib \
	icu \
	libunwind \
	base-devel \
 && pacman -Scc --noconfirm
 
# Install Rust toolchain via rustup
ENV RUSTUP_HOME=/root/.rustup \
	CARGO_HOME=/root/.cargo \
	PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:${DOTNET_INSTALL_DIR}
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
 && rustup default nightly

RUN cargo install sqlx-cli

RUN mkdir /migrations
COPY --from=builder /app/migrations /migrations

# Copy built binary and any required data
COPY --from=builder /app/target/release/nfmw-archive /usr/local/bin/nfmw-archive

EXPOSE 8074

ENV DOTNET_ROOT=/usr/share/dotnet
ENTRYPOINT ["/usr/local/bin/nfmw-archive"]


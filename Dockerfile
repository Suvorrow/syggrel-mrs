#Dockerfile
FROM rust:1.92-slim-trixie

# Install GTK3 dev dependencies (clean Ubuntu environment)
RUN apt update && \
    apt install -y \
        libgtk-3-dev \
        libcairo2-dev \
        libssl-dev \
        libwebkit2gtk-4.1-dev \ 
        libsoup-3.0-dev \
        python3-dev \
        python3-pip \
        python3-numpy \
        pkg-config \
        gcc \
        && rm -rf /var/lib/apt/lists/*

RUN pip3 install --break-system-packages gudhi

WORKDIR /app
COPY . .
RUN cargo build --release
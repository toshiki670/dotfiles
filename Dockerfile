# Development and testing environment for dotfiles
FROM ubuntu:24.04

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8
ENV TZ=UTC
ENV SHELL=/bin/zsh

# Install basic utilities and required packages
RUN apt-get update && apt-get install -y \
    curl \
    git \
    zsh \
    sudo \
    locales \
    ca-certificates \
    gnupg \
    lsb-release \
    build-essential \
    && locale-gen en_US.UTF-8 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install Homebrew (Linux)
RUN useradd -m -s /bin/zsh testuser \
    && echo 'testuser ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

USER testuser
WORKDIR /home/testuser

# Install Homebrew
RUN NONINTERACTIVE=1 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Add Homebrew to PATH
ENV PATH="/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:${PATH}"

# Install required tools via Homebrew
RUN brew install \
    gh \
    neovim \
    mise \
    sheldon \
    eza \
    bat \
    fd \
    ripgrep \
    zoxide \
    fzf \
    git-delta \
    chezmoi

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/testuser/.cargo/bin:${PATH}"

# Copy dotfiles to chezmoi source directory
COPY --chown=testuser:testuser . /home/testuser/.local/share/chezmoi

# Apply dotfiles with chezmoi
RUN chezmoi apply -v

# Set working directory to home
WORKDIR /home/testuser

# Set default shell to zsh
SHELL ["/bin/zsh", "-c"]

# Default command - start zsh as login shell
CMD ["/bin/zsh", "-l"]

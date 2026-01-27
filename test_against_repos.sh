#!/bin/bash

# Exit on any error
set -e

# Configuration
DART_UNUSED_DIR="$(pwd)"
TEMP_DIR="./temp_repos"
RESULTS_DIR="./results"

# Repository list and corresponding paths
REPOS=(
    "https://github.com/ReVanced/revanced-manager"
    "https://github.com/Predidit/Kazumi"
    # Add more repositories here
)

# Corresponding paths for each repository (empty string for root)
PATHS=(
    ""
    ""
    # Add corresponding paths here (must match REPOS array length)
)

# Flutter version to use
VERSION=(
    "3.29.3"
    "3.38.6"
    # Add more versions here (must match REPOS array length)
)

BUILD_COMMANDS=(
    "fvm dart run slang && fvm dart run build_runner build -d"
    "fvm dart run build_runner build -d"
    # Add more build commands here (must match REPOS array length)
)

# Validate arrays have same length
if [ ${#REPOS[@]} -ne ${#PATHS[@]} ] || [ ${#REPOS[@]} -ne ${#VERSION[@]} ] || [ ${#REPOS[@]} -ne ${#BUILD_COMMANDS[@]} ]; then
    echo "Error: REPOS, PATHS, VERSION, and BUILD_COMMANDS arrays must have the same length"
    exit 1
fi

# Create necessary directories
mkdir -p "$RESULTS_DIR"
mkdir -p "$TEMP_DIR"

# Function to extract repository name from URL
get_repo_name() {
    local repo_url="$1"
    echo "$repo_url" | sed 's/.*\/\([^\/]*\)\.git$/\1/' | sed 's/.*\/\([^\/]*\)$/\1/'
}

# Function to cleanup on exit
cleanup() {
    echo "Cleaning up temporary repositories..."
    rm -rf "$TEMP_DIR"
}

# Set trap to cleanup on script exit
trap cleanup EXIT

# Process each repository
for i in "${!REPOS[@]}"; do
    repo_url="${REPOS[$i]}"
    repo_path="${PATHS[$i]}"
    repo_name=$(get_repo_name "$repo_url")

    echo "=================================="
    echo "Processing repository: $repo_name"
    echo "URL: $repo_url"
    echo "Path: ${repo_path:-"(root)"}"
    echo "=================================="

    # Clone repository (shallow clone of default branch only)
    echo "Cloning repository..."
    cd "$TEMP_DIR"
    if ! git clone --depth 1 "$repo_url"; then
        echo "Error: Failed to clone $repo_url"
        exit 1
    fi

    # Go back to dart-unused directory
    cd "$DART_UNUSED_DIR"

    # Determine the target path
    target_path="$TEMP_DIR/$repo_name"
    if [ -n "$repo_path" ]; then
        target_path="$target_path/$repo_path"
    fi

    # Check if target path exists
    if [ ! -d "$target_path" ]; then
        echo "Error: Target path $target_path does not exist"
        exit 1
    fi
    cd "$target_path"
    echo "Current directory: $(pwd)"
    # Set Flutter version using fvm
    flutter_version="${VERSION[$i]}"
    fvm use "$flutter_version"
    echo "Using Flutter version: $(fvm flutter --version | head -n 1)"
    # Get dependencies
    fvm flutter pub get
    eval "${BUILD_COMMANDS[$i]}"

    # Return to dart-unused directory and run the tool
    cd "$DART_UNUSED_DIR"

    echo "Running dart-unused analysis..."
    result_file="$RESULTS_DIR/${repo_name}_results.txt"

    # Run dart-unused with specified arguments
    if ! cargo run --release -- --path "$target_path" -l -a > "$result_file" 2>&1; then
        echo "Error: dart-unused failed for $repo_name"
        exit 1
    fi

    echo "Results saved to: $result_file"
    echo "Analysis completed for $repo_name"
    echo ""

    # Clean up this repository before moving to next
    rm -rf "$TEMP_DIR/$repo_name"
done

echo "All repositories processed successfully!"
echo "Results are available in the $RESULTS_DIR directory:"
ls -la "$RESULTS_DIR"

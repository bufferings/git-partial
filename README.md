# GitPartial

> [!WARNING]
> This is my hobby project coding with Cursor and I haven't yet read any of the source codes Cursor wrote. Nothing is guaranteed.

GitPartial is a prototype tool designed to improve working with large Git repositories by utilizing sparse checkout. It allows you to clone only necessary parts of a repository and fetch updates relevant to those parts.

**Note:** This is currently a work-in-progress.

## Core Concepts

- **Partial Clone**: Instead of cloning the entire repository, you specify directories or file patterns to check out initially.
- **Metadata**: GitPartial stores metadata (`.gitpartial/metadata.json`) in the cloned repository root to track the original remote URL and the checked-out paths.
- **Sparse Checkout**: Leverages Git's built-in `sparse-checkout` feature.

## Implemented Commands

The following commands have basic implementations and passing acceptance tests:

- `clone <repo_url> <destination> --paths <path1> [path2...]`
  - Clones the specified `<repo_url>` into the `<destination>` directory.
  - Only the files matching the provided `--paths` (space-separated list of files or glob patterns) are checked out.
  - Creates a `.gitpartial/metadata.json` file.
- `add-paths <path1> [path2...]`
  - Run this command _inside_ a git-partial cloned repository.
  - Adds new paths to the sparse checkout definition.
  - Updates the working directory to include files matching the new paths.
  - Updates the `.gitpartial/metadata.json` file.
- `status`
  - Run this command _inside_ a git-partial cloned repository.
  - Displays the current branch, its status relative to the remote (`origin`), the last synced commit SHA, the remote URL, the list of currently checked-out sparse paths, and the output of `git status --short`.
- `smart-pull`
  - Run this command _inside_ a git-partial cloned repository.
  - Fetches changes from `origin` and performs a fast-forward merge (`git merge --ff-only origin/<current_branch>`).
  - Updates the last synced commit SHA in `.gitpartial/metadata.json`.
  - **Note:** This currently fetches all changes but relies on sparse-checkout to limit what affects the working directory. True "smart" fetching (only relevant objects) is not yet implemented.

## Usage Examples

```bash
# Clone only the 'src/frontend' directory and the main README
# Note: Destination directory must exist or be creatable
git-partial clone https://github.com/user/large-repo.git ./my-partial-repo --paths "src/frontend/**" README.md

# Navigate into the cloned repository
cd ./my-partial-repo

# Check the status
git-partial status

# Add the 'docs' directory to the checkout
git-partial add-paths "docs/**"

# Pull changes relevant to checked-out paths
git-partial smart-pull
```

## Development

```bash
# Clone this repository
git clone <your-fork-url>
cd git-partial

# Build
cargo build

# Run tests (all tests should pass)
cargo test
```

## License

MIT

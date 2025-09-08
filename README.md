# Taskfile mise Template

A Template for development with [Taskfile](https://taskfile.dev) and [mise](https://mise.jdx.dev).

## Getting started

1. Clone this repository.

    ```bash
    git clone git@github.com:23prime/taskfile-mise-template.git
    ```

2. Copy the cloned repository to anywhere..

    ```bash
    cp -ar taskfile-mise-template <your-repo-path>
    ```

3. Into your repository.

    ```bash
    cd <your-repo-path>
    ```

4. Rename remote repository to `upstream`.

    ```bash
    git remote rename origin upstream
    ```

5. Create your remote repository as `origin` and set URL, push.

    If you use GitHub CLI:

    ```bash
    gh repo create "<your-repo-name>" --private --source=. --remote=origin --push
    ```

    If you create repository on GitHub manually:

    ```bash
    git remote set-url origin <your-remote-url>
    git push -u origin main
    ```

6. Check remote repositories.

    ```bash
    $ git remote -v
    origin  <your-remote-url> (fetch)
    origin  <your-remote-url> (push)
    upstream        git@github.com:23prime/taskfile-mise-template.git (fetch)
    upstream        git@github.com:23prime/taskfile-mise-template.git (push)
    ```

## Merge from upstream

1. Fetch upstream changes.

    ```bash
    git fetch upstream
    ```

2. Merge.

    ```bash
    git merge upstream/main
    ```

# Development Template with mise

A Template for development with [mise](https://mise.jdx.dev).

## Getting started

1. Create new remote repository on GitHub.

2. Clone this repository.

    ```bash
    git clone git@github.com:23prime/mise-template.git
    ```

3. Copy the cloned repository to anywhere.

    ```bash
    cp -ar mise-template <new-repo-path>
    ```

4. Into new repository.

    ```bash
    cd <new-repo-path>
    ```

5. Rename remote repository to `upstream`.

    ```bash
    git remote rename origin upstream
    ```

6. Add new remote repository as `origin`.

    ```bash
    git remote add origin <new-remote-url>
    ```

7. Check remote repositories.

    ```bash
    $ git remote -v
    origin  <new-remote-url> (fetch)
    origin  <new-remote-url> (push)
    upstream        git@github.com:23prime/mise-template.git (fetch)
    upstream        git@github.com:23prime/mise-template.git (push)
    ```

8. Push to `origin`.

    ```bash
    git push -u origin main
    ```

9. If you use GitHub CLI, set the default repository.

    ```bash
    gh repo set-default <new-repo-name>
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

# CI/CD

**C**ontinuous **I**ntegration/**C**ontinuous **D**elivery is a conceptual approach that consists in automatizing actions done on applications throughout their life cycle such as tests, integration or delivery in order to accelerate their development.

For more information on the global topic, please refer to : <https://about.gitlab.com/topics/ci-cd/>

In the Tramex project, CI/CD is used for deployment and tests. There are three files :

- [`deploy.yml`](https://github.com/tramex/tramex/blob/main/.github/workflows/deploy.yml)
- [`tests.yml`](https://github.com/tramex/tramex/blob/main/.github/workflows/tests.yml)
- [`wasm.yml`](https://github.com/tramex/tramex/blob/main/.github/workflows/wasm.yml)

The files use the `.yml` format which is a human-readable serialization language. It is used to translate a data structure from a language to another.

The file gives precisions on the running context and the steps to follow. Each step has a name and corresponds to an action such as command execution.

## Deployment file

This file is executed when a push command is performed on the main branch of the GIT repository. It can be ran automatically when trying to push or manually from the Action tab.

The targets of the deployment file are the GitHub Pages. GitHub Pages is a static site hosting service that publishes a website using either HTML, CSS or JavaScript files stored on a GitHub repository. For more information on this topic, please refer to : <https://docs.github.com/en/pages/getting-started-with-github-pages/about-github-pages>

The deployment file follows the recommended steps which are the configuration of the pages, the cargo cache and the mdbook, the build, the coverage and finally the deployment.

In the Rust language, the `cargo` command corresponds to the `run` command. Once the script is executed, it builds the output in a specific location. In our case, the cargo cache is used for fastest results.

The website is built using the `trunk` command. It is important not to forget the `--release` flag in order to optimize the performances and the size of the output code. It is also necessary to use the `--public-url` flag because Tramex is stored on `https://tramex.github.io/tramex` domain (so in the folder `tramex/` - a relative url). To ease the deployment, all the built files are stored in the `dist` directory.

The coverage step corresponds to the measurement of the quantity of tested code. The output is automatically saved in a specific location but using the `rsync` command, the output is relocated in the `dist` directory as well.

In the Tramex application, it is possible to open documentation from the About section. This documentation appears as a website and is generated from the `mdbook`. `mdbook` is static site generator that publishes a website using `markdown` (`.md`) files. The `mdbook` is also set up, then it transforms all `.md` files located in the `docs` folder of the repository and output the result in the `dist` directory (as configured in the configuration file of `mdbook` (<https://github.com/tramex/tramex/blob/main/docs/book.toml>).

Finally the whole content of the `dist` directory is deployed to GitHub Pages.

After the deployment, the accessible URLs are:

- <https://tramex.github.io/tramex/> - browser version of the project
- <https://tramex.github.io/tramex/docs/> - user friendly documentation
- <https://tramex.github.io/tramex/coverage/> - coverage of the project
- <https://tramex.github.io/tramex/crates/tramex/> - developer friendly documentation of `tramex`
- <https://tramex.github.io/tramex/crates/tramex_tools/> - developer friendly documentation of `tramex_tools`

## Tests file

This file is executed when a push command is performed or when a pull is requested on the main branch of the Git repository. It is used to test code before any new modification is accepted in order to preserve the consistency of the Tramex code through the development.

The Tests file sets up the cargo cache, builds the project and runs the tests implemented in the `cargo test` command on the whole workspace. The `--verbose` flag is used to facilitate the debugging in the event of an error.

## WASM file

> WASM stands for **W**eb **AS**se**M**bly.

This file is executed when a push command is performed or when a pull is requested on the main branch of the Git repository. It is used to test code before any new modification is accepted in order to preserve the consistency of the Tramex code through the development. Unlike the Tests file that runs tests on the consistency of the global code, the WASM file specifically tests the build of the `wasm` rust target (made by `trunk`).

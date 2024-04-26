# CI/CD

**C**ontinuous **I**ntegration/**C**ontinuous **D**elivery is a conceptual approach that consists in automatizing actions done on applications throughout their life cycle such as tests, integration or delivery in order to accelerate their development.

For more information on the global topic, please refer to : <https://about.gitlab.com/topics/ci-cd/>

In the Tramex project, CI/CD is used for deployment, tests and web assembly. Therefore, there are three files :

- [`deploy.yml`](https://github.com/tramex/tramex/blob/main/.github/workflows/deploy.yml)
- [`tests.yml`](https://github.com/tramex/tramex/blob/main/.github/workflows/tests.yml)
- [`wasm.yml`](https://github.com/tramex/tramex/blob/main/.github/workflows/wasm.yml)

The files use the `.yml` format which is a human-readable serialization language. It is used to translate a data structure from a language to another.

The file gives precisions on the running context and the steps to follow. Each step has a name and corresponds to an action such as command execution.

## Deployment file

This file is executed when a push is performed on the main branch of the git repository. It can be ran automatically when trying to push or manually from the Action tab.



## Tests file



## WASM file
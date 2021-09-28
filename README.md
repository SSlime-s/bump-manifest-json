# bump-manifest-json
bump manifest.json version
## install
`cargo install --git https://github.com/SSlime-s/bump-manifest-json`
## usage
```
USAGE:
  manifest-bump [<version> | major | minor | patch] [FLAGS] [Options]

FLAGS:
  -g, --git  git commit and add tag
  -S         signature for git commit

OPTIONS:
  -r, --run <after-run>    run command after version bump (before git commit)
  -m, --message <message>  message for git commit (default: "📚 bump version v{version}")
```

# Syntagma is a manual

wrote by alexander14k

## TODO 🧙‍♀️🧙‍♂️📚✨✨

```git
# prepare a new ssh key (done once)
ssh-keygen -t ed25519 -C "your email here"

# start new ssh agent to prepare ssh for git pull private repo
eval $(ssh-agent -s)

# add your ssh key to the ssh agent session
ssh-add ~/.ssh/id_... 

# download remote private repo and use private ssh login password
git clone git@github.com:__some__user__/private_repo.git

# check your permissions to the remote repo
git remote -v

# check changes to the local repo
git status -sb

### ?? new changes
### A added changes
### AM added and new changes
## main...origin/main
?? syntagma/README.md
A  lexicon/README.md
AM syntagma/README.md

# add your work to prepare a new commit
git add README.md

# save your contributions on the local repo
git commit -m "add new_book"

# push to remote repo your contributions
git push -u origin main

# based on private or public specific repo use specific user and PAT password
```


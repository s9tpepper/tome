# Features

## TODO

### Release Checklist
[ ] Squash bugs for MVP
[x] Test on other terminals? (Ghostty, Kitty, Alacritty, iTerm, gnome terminal, konsole, terminator, xterm, gtk terminal)
[ ] Flesh out the README.md
[ ] Add GitHub Action(s) for producing binaries and release
[ ] Screenshots for the README
[x] Update the application name
[ ] Do a pass through the keybindings, make sure they make sense
[x] (MVP) Import to Postman format
[x] (MVP) Export to Postman format
[x] (MVP) saving endpoint/project is not updating the project/endpoint name display
[ ] (MVP) Update the code generating/response saving to go into ~/Documents/tome/<PROJECT_NAME>/<ACTION_TYPE>
[ ] (MVP) Add UI for choosing Body type (raw, form-data, etc)
[ ] (MVP) Add form-data handling
[ ] (MVP) Add file handling
[ ] (MVP) Add x-www-form-urlencoded handling
[ ] (MVP) Project/Request variables, like for tokens so you don't have to copy/paste tokens for every request
[ ] (MVP) Fix extension handling in requests.rs:92
[ ] (MVP) Pretty print responses that are minified

### Post-Release
[ ] Need a cool logo
[ ] github pages website/page?
[ ] Neovim wrapper plugin

### Bugs
[x] Endpoints Selector window is broken when the project doesn't have any endpoints
[x] top right menu goes offscreen when the project/endpoint names change
[x] URL input border is sometimes incorrect
[x] sort the app themes so they're always in the same order
[x] code gen window needs Esc to close the window
[x] fix the response filtering
[x] (MVP) reduce number of syntax highlighting themes, too many


### Options
[ ] (MVP?) Set up GitHub integration in the options?
[N] Allow keybinding customizations?? (not really sure about this one)
[x] Themes for UI colors

### UI
[x] Pass on cleaning up keybindings
[x] (MVP) Reserve Q for quitting app
[x] Pass on UI colors for readability
[x] (MVP) Fix bindings so they are contextual to the screen that you are on
[x] Clear project/endpoint name dialog inputs if the name is still default
[x] (MVP) Options screen to choose themes, background for responses/request bodies, etc
[x] (MVP) Flesh out the options for the app
[ ] Open In for URL/response
[x] (MVP) Fix Ctrl C in request body

### Response
Body
[x] (MVP) Save response body
[x] (MVP) Filter/Search response body
[x] (MVP) Highlight search results so you can see them as you navigate the search result list
[x] (MVP) Syntax Highlighting
[x] (MVP) Syntax Highlighting themes (tmTheme files)
[x] (MVP) Save new syntax themes to options
[x] (MVP) Update code sample and make window resize dynamically based on available space
[x] (MVP) Fix the entire response area background to be the intended bg color of the chosen theme
* [x] (MVP) Virtualized Response body view
[ ] Prettier/formatter type of integration to make responses read easier

### Projects
[x] (MVP) Switch projects
[x] (MVP) Switch endpoints
[x] (MVP) Add Project
[x] (MVP) Add Endpoint
[x] (MVP) Rename Project dialogue

### Requests
* [ ] (MVP?) Request dependencies/piping results from dependency
* [x] (MVP) Code generation (curl, TypeScript/JavaScript, Rust, PHP?, Go?, Python?)
[ ] Code generation plugin framework, based on OpenAPI code generation plugins/tools, maybe?
[ ] ***CODE GEN: Header Variables should become function arguments once variables are a thing in requests
[ ] Import/Export OpenAPI

### Text Input
[x] (MVP) Possible bug with backspace in text input not removing character

### Optimizations
[x] Update AppTheme/AppThemePersisted to use serde w/out derive to try to get rid of the need for two structs to save JSON
    Note: This didn't turn out so well, too complicated, kept duo structs


### Themes to Keep
- Bespin
- Blackboard Mod
- BlackLight
- Cobalt
- fake
- GlitterBomb

- Juicy (Light Theme)
- Midnight
- Monokai Dark
- Spectacular

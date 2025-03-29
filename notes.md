# Features

## TODO

[ ] Add mouse scroll up/down support in response renderer
[ ] Add mouse scroll up/down support in textarea component
[ ] Update layout/styling of syntax highlight selector window
[ ] (BUG) Fix focus when on Options screen and app loses/regains focus
[x] Mouse support for confirm action window
[x] Mouse support for Swap Project window
[x] Mouse support for Swap Endpoint window
[x] Mouse support for Body Mode area
[x] Mouse support for Method area
[x] Mouse support for clicking into URL input
[x] Mouse support for request body text area input
[ ] Mouse support for Commands Window options
[ ] Mouse scroll support for response renderer
[ ] Saving project after saving endpoint clears the project name and breaks the Project Name Edit window

### Release Checklist
[x] Flesh out the README.md
[x] Add GitHub Action(s) for producing binaries and release
[x] Screenshots for the README
[x] Do a pass through the keybindings, make sure they make sense
[x] Squash bugs for MVP
[x] Test on other terminals? (Ghostty, Kitty, Alacritty, iTerm, gnome terminal, konsole, terminator, xterm, gtk terminal)
[x] Update the application name
[x] (MVP) Import to Postman format
[x] (MVP) Export to Postman format
[x] (MVP) saving endpoint/project is not updating the project/endpoint name display
[x] (MVP) Update the code generating/response saving to go into ~/Documents/tome/<PROJECT_NAME>/<ACTION_TYPE>
[x] (MVP) Add UI for choosing Body type (raw, form-data, etc)
[x] (MVP) Add form-data handling
[x] (MVP) Add file handling
[x] (MVP) Add x-www-form-urlencoded handling
[x] (MVP) Fix extension handling in requests.rs:92
[x] (MVP) Fix selected/unselected colors of floating windows with text lists, like file selector, apply theme colors
[x] (MVP) Fix auto-focus for popups that have inputs, so the first input is automatically in focus
[x] (MVP) Update cursor in text inputs to use a color from the app theme
[x] (MVP) Refactor out old textinput components for the newer shared edit_input.rs
[x] (MVP) Style the Success/Error popups with app theme colors
[x] (MVP) Project/Request variables, like for tokens so you don't have to copy/paste tokens for every request
[x] (MVP) Project window add button
[x] (MVP) Project window delete button
[x] (MVP) Project window rename button doesn't do anything
[x] (MVP) Project window delete confirmation window needs style updates
[x] (MVP) Project window delete confirmation window No option doesn't work
[x] (MVP) Endpoints window Rename button doesn't do anything
[x] (MVP) Endpoints window Delete button tries to delete a project instead of the endpoint
[x] (MVP) Clear the list of endpoints when the project is swapped
[x] (MVP) Ensure that endpoint names are unique per project

[x] 100% scroll displays as 10%;
[x] Unique endpoint names are broken again (broken when changing names via N, change vs rename)
[x] Scrolling is not working correctly in endpoint names list

[IP] (POST-LAUNCH) Add Mouse Support
[ ] (POST-LAUNCH) Pretty print responses that are minified
[ ] (POST-LAUNCH) Fix code gen with variables
[ ] ***CODE GEN: Header Variables should become function arguments once variables are a thing in requests



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
[ ] (POST-LAUNCH) Set up GitHub integration in the options?
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
* [x] (MVP) Code generation (curl, TypeScript/JavaScript, Rust, PHP?, Go?, Python?)
[ ] Code generation plugin framework, based on OpenAPI code generation plugins/tools, maybe?
[IP] Import/Export OpenAPI

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

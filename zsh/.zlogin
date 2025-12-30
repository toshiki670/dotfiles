# Login config

# Ensure Homebrew paths come before system paths
# This runs before each prompt because mise reorders PATH
if [[ "$OSTYPE" == "darwin"* ]] && [[ -d "/opt/homebrew/bin" ]]; then
  _fix_homebrew_path() {
    # Only fix if Homebrew bin is not at the beginning
    if [[ "$PATH" != "/opt/homebrew/bin:"* ]]; then
      # Remove Homebrew paths from current PATH
      local new_path="$PATH"
      new_path=${new_path//\/opt\/homebrew\/bin:/}
      new_path=${new_path//\/opt\/homebrew\/sbin:/}
      new_path=${new_path//\/opt\/homebrew\/opt\/coreutils\/libexec\/gnubin:/}
      # Remove trailing duplicates if they exist at the end
      new_path=${new_path%:/opt/homebrew/bin}
      new_path=${new_path%:/opt/homebrew/sbin}
      new_path=${new_path%:/opt/homebrew/opt/coreutils/libexec/gnubin}
      # Add them at the front
      export PATH="/opt/homebrew/bin:/opt/homebrew/sbin:/opt/homebrew/opt/coreutils/libexec/gnubin:$new_path"
    fi
  }
  
  # Run once at login
  _fix_homebrew_path
  
  # Add to precmd_functions to run before each prompt
  precmd_functions+=(_fix_homebrew_path)
fi

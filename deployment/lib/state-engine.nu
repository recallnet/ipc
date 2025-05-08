# Runs steps and stores the state into the given file.
# Each step runs only once.
export def run [
  workdir: string,
  steps: list<record>, # list of records (name, fn), where fn takes $state as the argument and returns a state patch record.
  --log-prefix: string,
  ] {
  let workdir = ($workdir | path expand)
  let state_file = (state-file $workdir)
  let update = {|patch| update-state $workdir $patch}

  def log [str: string] {
    print $"(ansi '#f58c5f')== [step ($log_prefix | default "")] ($str)(ansi reset)"
  }

  $steps | each { |step|
    let state = (read-state $state_file)
    $env.state = ($state | merge {update: $update})
    if ($step.name not-in $state.completed_steps) {
      log $step.name
      do $step.fn $state
      update-state $workdir {completed_steps: ($state.completed_steps | insert $step.name true)}
      if ("ONE_STEP" in $env) {
        exit 0
      }
    }
  }
  log "== done =="
}

def read-state [state_file: string] {
  try {
    open $state_file
  } catch {
    {
      completed_steps: {}
    }
  }
}

export def update-state [workdir: string, patch: record] {
  mkdir $workdir
  let state_file = (state-file $workdir)
  read-state $state_file | merge deep --strategy=append $patch | save -f $state_file
}


def state-file [workdir: string] { $workdir | path join "state.yml" }

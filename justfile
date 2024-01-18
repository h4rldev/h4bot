default: deploy

@restart:
  cargo shuttle project restart --idle-minutes 0
  just deploy

@stop:
  cargo shuttle project stop

@start:
  cargo shuttle project start
  just deploy

@deploy:
  cargo shuttle deploy --no-test --allow-dirty

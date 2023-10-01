group "default" {
  targets = ["server"]
}


target "server" {
  context = "."
  target  = "server-stage"
  tags = ["cartesi/dapp:echo-devel-server"]
}
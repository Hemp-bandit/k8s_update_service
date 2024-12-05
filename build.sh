git_hash=$(git rev-parse --short=8 HEAD)
registry="registry.cn-hangzhou.aliyuncs.com/wyswill_docker"
pkg_name="kaibai_user_service"
token="eyJhbGciOiJSUzI1NiIsImtpZCI6IkpHNEtqci00NjcwWTRyQTEtaG10dnRJeGtoMlJyV2FsUk9GZFNJZEFHTTgifQ.eyJpc3MiOiJrdWJlcm5ldGVzL3NlcnZpY2VhY2NvdW50Iiwia3ViZXJuZXRlcy5pby9zZXJ2aWNlYWNjb3VudC9uYW1lc3BhY2UiOiJrdWJlLXN5c3RlbSIsImt1YmVybmV0ZXMuaW8vc2VydmljZWFjY291bnQvc2VjcmV0Lm5hbWUiOiJhZG1pbi11c2VyLXRva2VuLXJ3bThtIiwia3ViZXJuZXRlcy5pby9zZXJ2aWNlYWNjb3VudC9zZXJ2aWNlLWFjY291bnQubmFtZSI6ImFkbWluLXVzZXIiLCJrdWJlcm5ldGVzLmlvL3NlcnZpY2VhY2NvdW50L3NlcnZpY2UtYWNjb3VudC51aWQiOiJjMjdlMmYxMy01NjUwLTQ0ZGMtYjhkZi04ODE5MTliNzg1OGIiLCJzdWIiOiJzeXN0ZW06c2VydmljZWFjY291bnQ6a3ViZS1zeXN0ZW06YWRtaW4tdXNlciJ9.Fx0KBFd_mtqxTnsBK1QzySwh5_lKjL5CWzv6MzYIZd6Tc1yM6MdC-9c0AgR6oDvG_6XuATLaDpfXQ3rVb7UiIlkmqCwvOlwmqCb_qnqL0pCFbXTwoY5GFgaVv6YboKx6TE7m2j9OqA3_glZJtabfnG7GX_pPdyPoMOvvsPbRkdtohfMBG7w-tBIIQNqsjUuyFyEutpAL2Kp8FYqE0GcmFYVSiHLC6XAAWcPJxAMH6B-BnM1-4vNfS8FFN8wn-6SguhmkGu2sm2VCaO_4P3Y0ykr5YZn8aYzF-vz6R2ppYXkGZzNjLWAzYtCl1kC4NWibGIdh7UmM3qiXy7mTsn6t6Q"

export docker_tag="$registry/$pkg_name:$git_hash"
echo "docker_tag: $docker_tag"
function build_img() {
  echo "build img"
  docker build -t $docker_tag -f ./dockerfile .
}

function push_img() {
  echo "push img"
  docker push $docker_tag
}

function login_ali(){
  docker login -u=15717827650 -p wyswill4290  registry.cn-hangzhou.aliyuncs.com
}

function update_img() {
curl -k --request PATCH --url https://kaibai.cloud:6443/apis/apps/v1/namespaces/kaibai/deployments/kaibai-user-deployment --header "authorization: Bearer $token" --header 'content-type: application/strategic-merge-patch+json' --header 'user-agent: vscode-restclient' --data "{\"spec\": {\"template\": {\"spec\": {\"containers\": [{\"name\": \"kaibai-user-deploy\",\"image\": \"$docker_tag\"}]}}}}"
}

case $1 in
  "build_img")
    build_img
    ;;
  "push_img")
    push_img
    ;;
  "login_ali")
    login_ali
    ;;
  "update_img")
    update_img
    ;;
  *)
    echo "comd has push_img、build_img"
    ;;
esac
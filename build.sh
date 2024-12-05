git_hash=$(git rev-parse --short=8 HEAD)
registry="registry.cn-hangzhou.aliyuncs.com/wyswill_docker"
pkg_name="kaibai_user_service"

export docker_tag="$registry/$pkg_name:$git_hash"
echo "docker_tag: $docker_tag"

case $1 in
"build_img")
  docker build -t $docker_tag -f ./dockerfile .
  ;;
"push_img")
  docker push $docker_tag
  ;;
"login_ali")
  docker login -u=15717827650 -p wyswill4290 registry.cn-hangzhou.aliyuncs.com
  ;;
*)
  echo "comd has push_img、build_img"
  ;;
esac

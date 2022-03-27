# Finance assistant based on rust

基于 CS 架构的财务助手服务端 api 搭建,采用了 tide 框架,数据库连接使用了 sqlx(曾采用过 sea-orm 但在最终环境部署时未能正常运作),同时使用了 rust-crypto 加密库为用户登录的状态信息做了完备的加密与检查

该项目作为完整项目的后端,android 前端请参见[这里](https://github.com/san-qi/Finance_client)

## 项目结构

```
.
├── Cargo.lock
├── Cargo.toml
├── README.md
└── src
    ├── main.rs
    ├── route
    │   ├── mod.rs
    │   ├── prelude.rs
    │   ├── record.rs
    │   └── user.rs
    └── util
        ├── check_login.rs
        ├── cryp.rs
        ├── get_json.rs
        ├── mod.rs
        ├── prelude.rs
        └── regex_check_format.rs
```

- route
  - user: api 路由逻辑
  - record: api 路由逻辑
- util
  - check_login: 检查登录
  - cryp: aes 算法加密
  - regex_check_format: regex 正则匹配

## 接口格式

- 请求

  - localhost:8084/register -d '{"uid":?, "password":"?", "email":"?"}'
  - localhost:8084/login -d '{"uid":?, "password":"?"}'
  - localhost:8084/password -d '{"password":"?"}' --cookie "uid=?;info=?"
  - localhost:8084/upload -d '[{"id":?, "amount":?, "date":"?", "type":"?", "isIncome":?}]' --cookie "uid=?;info=?"
  - localhost:8084/delete?rid=0 -d '[]' --cookie "uid=?;info=?"
  - localhost:8084/download?rid=0 --cookie "uid=?;info=?"

- 返回:
  - {"code":?, "data":[], "details":"?"}
  - --setcookie "uid=?;info=?"

## 授权许可

请遵循[GNU GPLv3](https://www.gnu.org/licenses/gpl-3.0.html)开源许可,上传到 github 仅仅是为了记录,这也算是大四学期的用心之作了

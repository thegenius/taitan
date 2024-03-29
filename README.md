<div align='center'> <h1>Great Art Stretches Taste, It Doesn’t Follow Tastes.</h1></div>


# Features

- **Modern**    
[1] HTTPS First: http request will redirect to https with **301(permanent)**  
[2] Graceful Shutdown and Graceful startup.  
[3] Best Practice: default config.(listen to 0.0.0.0:443 for example)  
[4] Tracing request

- <h2>Ergonomics Result Flow</h2>  
[1] Delicately Designed Response 
``` 
  200/OK
  {
    success : true,
    data: T
  }
  
  200/OK 
  {
    success: false,
    err_code: u32,
    err_msg: String
  }
```
  -  **Why not restful http status code?**  
Because http status code is for resource, for example *.jpg, *.html,  
RPC is not resource, it is remote call, return a reponse with err_msg.  
People try to merge RPC to HTTP time after time.  
And every time I saw mess up.  
So **BEST PRACTICE** is make them do their own bessiness.  
Return with false + err_code + err_msg.  
- false indicate there is an error,  
- err_code indicate the category of error,  
- err_msg indicate the concrete err msg, may be you can show to user.  

This design has been tortured by huge amout of system,  
and proved itself.

Why need three field to indicate one error?
- false is used by the control flow and error system
  - error system may throw exception
  - control flow may jump to error handler
- err_code is used by error handler logic
  - almost every time your handle logic need to know error category
- err_msg is the info you should report to user or developer
  - logic or control flow should not depend on msg


[1] Explicit Error: every error handle with thiserror::Error  
[2] Ergonomics Error classification  
  -  Recoverable Error: usually I/O error(database/connection)  
  -  System Error: Report to Admin, Parse Error, unexpected bug and else  
  -  Logic Error: Report to User, password error and so on  
For example  
[1] user input a mess string as input, cause a param Deserialize error, this is logic error    
[2] user input a valid string as input, but parse error unexpected, this is system error  
[3] database connection timeout is recoverable error  
[4] restful timeout is recoverable error  
[5] primary key duplicated is logic error(if api is idempotent, it may treat as success)


- **Modern SPA Support**  
[1] serve spa files  
[2] serve spa routing  
[3] SEO  

- **Security**  
[1] server security  
[1.1] request based limit  
[1.2] path based request limit  
[2] ip security  
[2.1] ip segment forbid  
[2.2] exact ip forbid  
[3] user security  
[3.1] user-id based forbid  
[3.2] user-id based request limit  

- **Human Checking**  
automated human checking with no pain  

- **Anthentication**(verifying who a user is)    
Every request auth must check by redis or database is a heavy burden for server.  
JWT force the client to prove themselves, this is the right way, but what about Logout.  
JWT, not redis or database based. Only use database + mem-cache for logout.  
[1] check who are you! forbid almost all api-endpoint expect specific ones.    
[2] Modern single Sign-On(SSO) with iframe based way, without ancient cookies.    
[3] register and login/logout    

- **Authorization**(verifying what they have access to)    
classic Role-Based-Access-Control(RBAC) is designed for **Enterprise**, not open community.  
For normal customers, Resource-Based-Access is the right policy.  
[1] super powerful api-endpoint limited to special roles(admin or system)    
[2] normal user live with resource-based access control  
[2.1] write control   
[2.2] read control  


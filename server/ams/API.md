# API 文档

## 认证
    API请求在header中携带Token

## 配方管理

### 1、获取配方列表

GET /recipes?tool_name=设备A&status=active&page=1&page_size=20   
  
**响应示例**:
```json
{
  "data": [
    {
      "recipe_id": 1,
      "tool_name": "设备A",
      "recipe_name": "工艺1", 
      "recipe_version": "1.0",
      "status": "active",
      "created_by": "user1"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total_count": 1
  }
}
```

### 2、创建配方
POST /recipes/create
请求体：
```json
{
  "tool_name": "设备A",
  "recipe_name": "工艺1",
  "recipe_version": "1.0",
  "inputs": ["原料 100", "原料 200"],
  "inputbuss": []
}
```
响应体：
```json
{
  "tool_name": "设备A",
  "recipe_name": "工艺1",
  "recipe_version": "2.0",
  "status": "active",
  "inputs": ["原料 100", "原料 200"],
  "inputbuss": []
}

{
  "error": "设备 [CR1] 上已存在配方 [水] 版本 [1.0]" 
}
```

### 3、更新配方
POST /recipes/update
请求体：
```json
{
  "tool_name": "设备A",
  "recipe_name": "工艺1",
  "recipe_version": "1.0",
  "inputs": ["原料 100", "原料 200"],
  "inputbuss": []
}
```
响应体：
```json
{
  "tool_name": "设备A",
  "recipe_name": "工艺1",
  "recipe_version": "2.0",
  "inputs": ["原料 100", "原料 200"],
  "inputbuss": []
}

{
  "error": "设备 [CR1] 上配方 [水] 不存在" 
}
```

### 4、切换配方版本
POST /recipes/switch-version
```json
{
  "tool_name": "CR1",
  "recipe_name": "水",
  "new_version": "2.1",
}
```

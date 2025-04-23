# Model Context Protocol: Theater Server Specification

## 1. Overview

The Theater MCP Server provides a standardized interface for language models and MCP clients to interact with the Theater WebAssembly actor system. This specification defines the capabilities, resources, tools, and interaction patterns supported by the integration.

## 2. Server Information

- **Name**: `theater-mcp`
- **Version**: `0.1.0`
- **Protocol Version**: `2024-11-05`
- **Theater Connection**: TCP connection to `localhost:9000`

## 3. Capabilities

The Theater MCP Server implements the following MCP capabilities:

```json
{
  "capabilities": {
    "resources": {
      "listChanged": true
    },
    "tools": {
      "listChanged": true
    }
  }
}
```

## 4. Resources

### 4.1 Actor Resources

#### 4.1.1 Actor List
- **URI**: `theater://actors`
- **Description**: List of all running actors in the Theater system
- **MIME Type**: `application/json`
- **Content Structure**:
  ```json
  {
    "actors": [
      {
        "id": "actor_uuid",
        "name": "Actor Name",
        "status": "RUNNING",
        "uri": "theater://actor/actor_uuid"
      }
    ],
    "total": 10
  }
  ```

#### 4.1.2 Actor Details
- **URI Template**: `theater://actor/{actor_id}`
- **Description**: Detailed information about a specific actor
- **MIME Type**: `application/json`
- **Content Structure**:
  ```json
  {
    "id": "actor_uuid",
    "manifest": "path/to/manifest",
    "status": "RUNNING",
    "created_at": "2025-04-23T14:30:00Z",
    "events_uri": "theater://events/actor_uuid",
    "state_uri": "theater://actor/actor_uuid/state"
  }
  ```

#### 4.1.3 Actor State
- **URI Template**: `theater://actor/{actor_id}/state`
- **Description**: Current state of a specific actor
- **MIME Type**: `application/json`
- **Content Structure**: Actor-specific state data (JSON)

### 4.2 Event Resources

#### 4.2.1 Actor Events
- **URI Template**: `theater://events/{actor_id}`
- **Description**: Event chain for a specific actor
- **MIME Type**: `application/json`
- **Content Structure**:
  ```json
  [
    {
      "event_id": "event_uuid",
      "actor_id": "actor_uuid",
      "timestamp": "2025-04-23T14:30:00Z",
      "type": "StateChanged",
      "data": { ... }
    },
    ...
  ]
  ```

## 5. Tools

### 5.1 Actor Management Tools

#### 5.1.1 Start Actor
- **Name**: `start_actor`
- **Description**: Start a new actor from a manifest
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "manifest": {
        "type": "string",
        "description": "Path to the actor manifest or manifest content"
      },
      "initial_state": {
        "type": "object",
        "description": "Optional initial state for the actor"
      }
    },
    "required": ["manifest"]
  }
  ```
- **Output**: Information about the started actor including its ID

#### 5.1.2 Stop Actor
- **Name**: `stop_actor`
- **Description**: Stop a running actor
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "actor_id": {
        "type": "string",
        "description": "ID of the actor to stop"
      }
    },
    "required": ["actor_id"]
  }
  ```
- **Output**: Confirmation of actor termination

#### 5.1.3 Restart Actor
- **Name**: `restart_actor`
- **Description**: Restart a running actor
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "actor_id": {
        "type": "string",
        "description": "ID of the actor to restart"
      }
    },
    "required": ["actor_id"]
  }
  ```
- **Output**: Confirmation of actor restart

### 5.2 Actor Communication Tools

#### 5.2.1 Send Message
- **Name**: `send_message`
- **Description**: Send a message to an actor
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "actor_id": {
        "type": "string",
        "description": "ID of the actor to send the message to"
      },
      "data": {
        "type": "string",
        "description": "Message data (base64 encoded)"
      }
    },
    "required": ["actor_id", "data"]
  }
  ```
- **Output**: Confirmation of message delivery

#### 5.2.2 Request Message
- **Name**: `request_message`
- **Description**: Send a request to an actor and receive a response
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "actor_id": {
        "type": "string",
        "description": "ID of the actor to send the request to"
      },
      "data": {
        "type": "string",
        "description": "Request data (base64 encoded)"
      }
    },
    "required": ["actor_id", "data"]
  }
  ```
- **Output**: Response data from the actor

### 5.3 Channel Tools

#### 5.3.1 Open Channel
- **Name**: `open_channel`
- **Description**: Open a communication channel to an actor
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "actor_id": {
        "type": "string",
        "description": "ID of the actor to open a channel with"
      },
      "initial_message": {
        "type": "string",
        "description": "Initial message data (base64 encoded)"
      }
    },
    "required": ["actor_id"]
  }
  ```
- **Output**: Channel ID and confirmation

#### 5.3.2 Send on Channel
- **Name**: `send_on_channel`
- **Description**: Send a message on an open channel
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "channel_id": {
        "type": "string",
        "description": "ID of the channel"
      },
      "message": {
        "type": "string",
        "description": "Message data (base64 encoded)"
      }
    },
    "required": ["channel_id", "message"]
  }
  ```
- **Output**: Confirmation of message sent

#### 5.3.3 Close Channel
- **Name**: `close_channel`
- **Description**: Close an open channel
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "channel_id": {
        "type": "string",
        "description": "ID of the channel to close"
      }
    },
    "required": ["channel_id"]
  }
  ```
- **Output**: Confirmation of channel closure

## 6. Implementation Notes

### 6.1 Simplified Approach
- **Transactional Model**: All interactions are request/response based
- **No Subscriptions**: Initial implementation does not rely on Theater subscription system
- **Error Handling**: Errors are reported directly in response to requests
- **JSON Conversion**: All Theater state and messages are treated as JSON

### 6.2 Theater Connection
- Connection to Theater server at `localhost:9000`
- TCP socket-based communication
- All Theater commands are proxied via Theater JSON protocol

### 6.3 Actor Identity
- Actor IDs use full UUID format as generated by Theater
- No additional mapping or friendly naming in initial implementation

### 6.4 Future Enhancements
- Remote Theater server connections
- Subscription-based updates
- Friendly naming for actors
- Authentication and authorization

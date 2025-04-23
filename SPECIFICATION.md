# Model Context Protocol: Theater Server Specification

## 1. Overview

The Theater MCP Server provides a standardized interface for language models and MCP clients to interact with the Theater WebAssembly actor system. This specification defines the capabilities, resources, tools, and interaction patterns supported by the integration.

## 2. Server Information

- **Name**: `theater-mcp`
- **Version**: `0.1.0`
- **Protocol Version**: `2024-11-05`
- **Default Connection**: TCP socket or stdio transport

## 3. Capabilities

The Theater MCP Server implements the following MCP capabilities:

```json
{
  "capabilities": {
    "resources": {
      "subscribe": true,
      "listChanged": true
    },
    "tools": {
      "listChanged": true
    },
    "prompts": {
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
        "id": "actor_id",
        "name": "Actor Name",
        "status": "RUNNING",
        "uri": "theater://actor/actor_id"
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
    "id": "actor_id",
    "manifest": "path/to/manifest",
    "status": "RUNNING",
    "created_at": "2025-04-23T14:30:00Z",
    "events_uri": "theater://events/actor_id",
    "state_uri": "theater://actor/actor_id/state",
    "metrics": { ... }
  }
  ```

#### 4.1.3 Actor State
- **URI Template**: `theater://actor/{actor_id}/state`
- **Description**: Current state of a specific actor
- **MIME Type**: `application/json`
- **Content Structure**: Actor-specific state data

### 4.2 Event Resources

#### 4.2.1 Actor Events
- **URI Template**: `theater://events/{actor_id}`
- **Description**: Event chain for a specific actor
- **MIME Type**: `application/json`
- **Content Structure**:
  ```json
  [
    {
      "event_id": "event_123",
      "actor_id": "actor_id",
      "timestamp": "2025-04-23T14:30:00Z",
      "type": "StateChanged",
      "parent_id": null,
      "data": { ... },
      "children": [
        {
          "id": "event_124",
          "uri": "theater://event/event_124"
        }
      ]
    },
    ...
  ]
  ```

#### 4.2.3 Event by ID
- **URI Template**: `theater://event/{event_id}`
- **Description**: Specific event by its unique ID
- **MIME Type**: `application/json`
- **Content Structure**:
  ```json
  {
    "event_id": "event_123",
    "actor_id": "actor_id",
    "timestamp": "2025-04-23T14:30:00Z",
    "type": "StateChanged",
    "parent_id": "event_122",
    "data": { ... },
    "children": [
      {
        "id": "event_124",
        "uri": "theater://event/event_124"
      }
    ]
  }
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

## 6. Subscriptions and Notifications

### 6.3 List Changed Notifications
- `notifications/resources/list_changed`: Sent when the list of actors changes

## 7. Implementation Notes

### 7.1 Theater Client
- Connections to Theater server are maintained via TCP
- All operations are proxied through to the Theater server
- Client maintains a mapping between MCP resources and Theater entities

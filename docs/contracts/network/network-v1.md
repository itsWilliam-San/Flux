# Network IPC Specification

## Overview
How to transmit sewer network system data between processes.

## Units
All measurement units (Terrain Level, Invert Level, Depth, Slope, Incremental Length, Length, Diameter, and others) use **meters** or **square meters**.

---

## Data Entities

### 1. Intermediate Station
Represents an evenly spaced intermediate point along a segment.

- **ID**: Unique identifier string.
- **TerrainLevel**: Ground surface elevation.
- **InvertLevel**: Inner bottom elevation of the conduit/channel.
- **Depth**: Vertical distance from the terrain level to the invert level.

### 2. Station
Represents a manhole or any primary structure that behaves like one.

- **ID**: Unique identifier string.
- **TerrainLevel**: Ground surface elevation of the structure.
- **InvertLevel**: Inner bottom elevation of the structure.
- **Depth**: Difference between `TerrainLevel` and `InvertLevel`.
- **UpstreamManholes**: List of incoming manhole IDs connected from upstream.
- **psets**: A categorized key-value store designed for extensible plugin metadata.
  Properties are grouped by domain/category to prevent naming collisions and for
  interoperability between analysis tools.

#### DownstreamConnection
Describes both the target structure and how this station connects to it.

- **ID**: ID of the downstream manhole it connects to.
- **SegmentID**: The physical pipe span ID leading to the downstream connection.

#### Segment
- **ID**: Unique identifier of the section.
- **Pipe**: Material and dimensions spec.
    - **Material**: Pipe material (e.g., PVC, Ductile Iron).
    - **Length**: Total length of the pipe.
    - **Slope**: Gradient/slope of the pipe.
    - **Diameter**: Internal pipe diameter.
- **Intermediates**: List of IDs referencing all intermediate stations within this segment.
- **IntermediateDistance**: Positional distance from one intermediate station to another.
- **Connection**: Type of connection (e.g., `StepDrop`) and **InvertLevel** of the end of the Pipe.

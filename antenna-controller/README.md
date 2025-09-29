# AntennaController `send` Method

The `send` method of the `AntennaController` struct is used to transmit antenna control data over a serial port.

## Parameters

- **azimuth (`f64`)**: The azimuth angle (in degrees) to which the antenna should point.
- **elevation (`f64`)**: The elevation angle (in degrees) to which the antenna should point.
- **sat_name (`&str`)**: The name or identifier of the satellite being tracked.
- **downlink_number (`i64`)**: The downlink frequency or channel number associated with the satellite.

## Data Format

The data is sent as a single line of text in the following format:

```
SN=[sat_name],AZ=[azimuth],EL=[elevation],DN=[downlink_number]
```

- `SN` is the satellite name or identifier.
- `AZ` is the azimuth angle, formatted with one decimal place.
- `EL` is the elevation angle, formatted with one decimal place.
- `DN` is the downlink number.

**Example:**

If you call:

```rust
controller.send(123.4, 45.6, "ISS", 145800)
```

The following line will be sent over the serial port:

```
SN=ISS,AZ=123.4,EL=45.6,DN=145800
```

> **Note:**
> This format must be kept for compatibility with the existing system in the antenna. It ensures that the antenna can correctly interpret the commands sent to it.

# Overview
Nif (New Image Format) is an extremely simple media format for storing images and videos losslessly. It is designed to be straightforward to use, understand and implement. A serializer and deserializer can be trivially implemented within less than 200 lines of code, depending upon the language of choice. Nif files have .nif (for uncompressed images), .nvf (for uncompressed videos), .nifz (for compressed images) and .nvfz (for compressed videos) extensions. Currently,no compression is supported. Importantly, all nif data is serialized in Big-Endian (network ordering).

## Structure
Nif is a simple, flat file format encoded in binary. It is structured as follows:
1. Magic number: 4 bytes, always 0x4E-49-46-00 (NIF)
2. Version: 4 bytes, the current version of the file format is 0x00-01-00-00 (0.1.0) Follows standard versioning rules.
3. Feature flags: 4 bytes, currently unused. Will be used to indicate features of the file format (compression, etc.)
3. Header: The header is always present, and contains the following fields:
    - Image width: 4 bytes, the width of the image in pixels (int32).
    - Image height: 4 bytes, the height of the image in pixels (int32).
    - Pixel storage format: 4 bytes, the format of the image (int32). Can be one of the following:
        - 0: RGBA8888I32 (RGBA 8-bit per pixel, 32-bit integer)
        - 1: RGB888I32 (RGB 8-bit per pixel)
        - 2: RGBA444I16 (RGBA 4-bit per pixel, 16-bit integer)
        - 3: RGB444I16 (RGB 4-bit per pixel)
       
    - Frame count: 4-bytes. The number of frames in the file. Stored as an unsigned 32-bit integer. If the file contains a single frame, this field is 0, and it may be treated as an image.
    - Fps: The number of frames per second the file was recorded at. Encoded as a 32bit float.
    - Frame data: The frame data. The format of the frame data is determined by the pixel storage format. The frame data is stored contiguously, with no padding. The size of the frame data is determined by the width, height and pixel storage format, which can be trivially calculated from the header.

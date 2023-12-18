//! TUS protocol extensions
//!
//! Clients and Servers are encouraged to implement as many of the extensions as possible. Feature
//! detection SHOULD be achieved by the Client sending an OPTIONS request and the Server responding
//! with the Tus-Extension header.
//!
//! See [TUS protocol extensions](https://tus.io/protocols/resumable-upload.html#protocol-extensions)

use std::fmt::Display;

use headers::ContentType;

use crate::tus::headers::{
    TusExtensionHeader,
    UploadOffsetHeader,
};

pub enum Extension {
    /// The Client and the Server SHOULD implement the upload creation extension. If the Server
    /// supports this extension, it MUST add creation to the Tus-Extension header.
    Creation,
    /// The Client MAY include parts of the upload in the initial Creation request using
    /// [Extension::CreationWithUpload].
    ///
    /// If the Server supports this extension, it MUST advertise this by including
    /// `creation-with-upload` in the [TusExtensionHeader] header. Furthermore, this extension
    /// depends directly on [Extension::Creation]. Therefore, if the Server does not offer the
    /// Creation extension, it MUST NOT offer the Creation With Upload extension either.
    ///
    /// The Client MAY include either the entirety or a chunk of the upload data in the body of the
    /// POST request. In this case, similar rules as for the PATCH request and response apply. The
    /// Client MUST include the [ContentType]: `application/offset+octet-stream` header.
    /// The Server SHOULD accept as many bytes as possible and MUST include the
    /// [UploadOffsetHeader] header in the response and MUST set its value to the offset of the
    /// upload after applying the accepted bytes.
    ///
    /// If the Client wants to use this extension, the Client SHOULD verify that it is supported by
    /// the Server before sending the POST request. In addition, the Client SHOULD include the
    /// Expect: 100-continue header in the request to receive early feedback from the Server on
    /// whether it will accept the creation request, before attempting to transfer the first chunk.
    CreationWithUpload,
    /// The Server MAY remove unfinished uploads once they expire. In order to indicate this
    /// behavior to the Client, the Server MUST add expiration to the [TusExtensionHeader] header.
    Expiration,
    /// The Client and the Server MAY implement and use this extension to verify data integrity of
    /// each PATCH request. If supported, the Server MUST add checksum to the [TusExtensionHeader]
    /// header.
    ///
    /// A Client MAY include the Upload-Checksum header in a PATCH request. Once the entire request
    /// has been received, the Server MUST verify the uploaded chunk against the provided checksum
    /// using the specified algorithm. Depending on the result the Server MAY respond with one of
    /// the following status code:
    ///
    /// 1. `400 Bad Request` if the checksum algorithm is not supported by the server,
    /// 2. `460 Checksum Mismatch` if the checksums mismatch or
    /// 3. `204 No Content` if the checksums match and the processing of the data succeeded. In the
    /// first two cases the uploaded chunk MUST be discarded, and the upload and its offset MUST
    /// NOT be updated. The Server MUST support at least the SHA1 checksum algorithm identified
    /// by sha1. The names of the checksum algorithms MUST only consist of ASCII characters with
    /// the modification that uppercase characters are excluded.
    ///
    /// The Tus-Checksum-Algorithm header MUST be included in the response to an OPTIONS request.
    ///
    /// If the hash cannot be calculated at the beginning of the upload, it MAY be included as a
    /// trailer. If the Server can handle trailers, this behavior MUST be announced by adding
    /// checksum-trailer to the [TusExtensionHeader] header. Trailers, also known as trailing
    /// headers, are headers which are sent after the request’s body has been transmitted
    /// already. Following RFC 7230 they MUST be announced using the Trailer header and are
    /// only allowed in chunked transfers.
    Checksum,
    /// This extension defines a way for the Client to terminate completed and unfinished uploads
    /// allowing the Server to free up used resources.
    ///
    /// If this extension is supported by the Server, it MUST be announced by adding termination to
    /// the [TusExtensionHeader] header.
    Termination,
    /// This extension can be used to concatenate multiple uploads into a single one enabling
    /// Clients to perform parallel uploads and to upload non-contiguous chunks. If the Server
    /// supports this extension, it MUST add concatenation to the Tus-Extension header.
    ///
    /// A partial upload represents a chunk of a file. It is constructed by including the
    /// Upload-Concat: partial header while creating a new upload using the Creation extension.
    /// Multiple partial uploads are concatenated into a final upload in the specified order. The
    /// Server SHOULD NOT process these partial uploads until they are concatenated to form a final
    /// upload. The length of the final upload MUST be the sum of the length of all partial
    /// uploads.
    ///
    /// In order to create a new final upload, the Client MUST add the Upload-Concat header to the
    /// upload creation request. The value MUST be final followed by a semicolon and a
    /// space-separated list of the partial upload URLs that need to be concatenated. The partial
    /// uploads MUST be concatenated as per the order specified in the list. This concatenation
    /// request SHOULD happen after all of the corresponding partial uploads are completed. The
    /// Client MUST NOT include the Upload-Length header in the final upload creation.
    ///
    /// The Client MAY send the concatenation request while the partial uploads are still in
    /// progress. This feature MUST be explicitly announced by the Server by adding
    /// concatenation-unfinished to the Tus-Extension header.
    ///
    /// When creating a new final upload the partial uploads’ metadata SHALL NOT be transferred to
    /// the new final upload. All metadata SHOULD be included in the concatenation request using
    /// the Upload-Metadata header.
    ///
    /// The Server MAY delete partial uploads after concatenation. The Client, however, MAY attempt
    /// to use a partial upload multiple times. The same partial upload MAY be present multiple
    /// times in the Upload-Concat header in one upload creation request or MAY be used in multiple
    /// upload creation requests.
    ///
    /// The Server MUST respond with the 403 Forbidden status to PATCH requests against a final
    /// upload URL and MUST NOT modify the final or its partial uploads.
    ///
    /// The response to a HEAD request for a final upload SHOULD NOT contain the Upload-Offset
    /// header unless the concatenation has been successfully finished. After successful
    /// concatenation, the Upload-Offset and Upload-Length MUST be set and their values MUST be
    /// equal. The value of the Upload-Offset header before concatenation is not defined for a
    /// final upload.
    ///
    /// The response to a HEAD request for a partial upload MUST contain the Upload-Offset header.
    ///
    /// The Upload-Length header MUST be included if the length of the final resource can be
    /// calculated at the time of the request. Response to HEAD request against partial or final
    /// upload MUST include the Upload-Concat header and its value as received in the upload
    /// creation request.
    Concatenation,
}

impl Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Extension::Creation => write!(f, "creation"),
            Extension::CreationWithUpload => write!(f, "creation-with-upload"),
            Extension::Expiration => write!(f, "expiration"),
            Extension::Checksum => write!(f, "checksum"),
            Extension::Termination => write!(f, "termination"),
            Extension::Concatenation => write!(f, "concatenation"),
        }
    }
}

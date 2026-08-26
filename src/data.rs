#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Room {
    pub number: &'static str,
    pub room_type: &'static str,
    pub floor: u32,
    pub capacity: u32,
    pub price: u32,
    pub amenities: &'static [&'static str],
    pub status: &'static str,
    pub discount_percent: u32,
    pub breakfast_included: bool,
    pub uuid: &'static str,
    pub created_at: &'static str,
    pub updated_at: &'static str,
}

pub const ROOMS: &[Room] = &[
    Room { number: "101", room_type: "Standard Room", floor: 1, capacity: 2, price: 4500, amenities: &["Wi-Fi", "TV", "AC"], status: "Available", discount_percent: 10, breakfast_included: true, uuid: "a3f2b6c7-9d11-4e8a-bb1f-7c0d9e1f2a34", created_at: "May 12, 2024 10:30 AM", updated_at: "May 12, 2024 10:30 AM" },
    Room { number: "102", room_type: "Deluxe Room", floor: 1, capacity: 2, price: 6200, amenities: &["Wi-Fi", "TV", "AC", "Minibar"], status: "Occupied", discount_percent: 0, breakfast_included: true, uuid: "b4a1c8d2-1e23-4f9b-8c2a-5d6e7f809b12", created_at: "Mar 3, 2024 09:15 AM", updated_at: "May 10, 2024 04:20 PM" },
    Room { number: "201", room_type: "Twin Room", floor: 2, capacity: 2, price: 5100, amenities: &["Wi-Fi", "TV", "Breakfast"], status: "Reserved", discount_percent: 5, breakfast_included: true, uuid: "c5b2d9e3-2f34-4a0c-9d3b-6e7f8091c223", created_at: "Jan 20, 2024 11:00 AM", updated_at: "Apr 28, 2024 02:45 PM" },
    Room { number: "301", room_type: "Executive Suite", floor: 3, capacity: 4, price: 9800, amenities: &["Wi-Fi", "TV", "AC", "Minibar"], status: "Maintenance", discount_percent: 0, breakfast_included: true, uuid: "d6c3e0f4-3045-4b1d-ae4c-7f809a12d334", created_at: "Nov 8, 2023 08:30 AM", updated_at: "May 15, 2024 09:00 AM" },
    Room { number: "104", room_type: "Family Room", floor: 1, capacity: 4, price: 7400, amenities: &["Wi-Fi", "TV", "AC"], status: "Available", discount_percent: 15, breakfast_included: false, uuid: "e7d4f105-4156-4c2e-bf5d-809b1c23e445", created_at: "Feb 14, 2024 01:20 PM", updated_at: "May 2, 2024 10:10 AM" },
    Room { number: "205", room_type: "Single Room", floor: 2, capacity: 1, price: 3900, amenities: &["Wi-Fi", "Desk", "AC"], status: "Available", discount_percent: 0, breakfast_included: false, uuid: "f8e50216-5267-4d3f-c06e-91ac2d34f556", created_at: "Dec 1, 2023 03:45 PM", updated_at: "Apr 20, 2024 12:30 PM" },
    Room { number: "202", room_type: "Deluxe Room", floor: 2, capacity: 2, price: 6200, amenities: &["Wi-Fi", "TV", "AC"], status: "Occupied", discount_percent: 10, breakfast_included: true, uuid: "09f61327-6378-4e40-d17f-a2bd3e45f667", created_at: "Jan 9, 2024 09:50 AM", updated_at: "May 14, 2024 06:05 PM" },
    Room { number: "401", room_type: "Executive Suite", floor: 4, capacity: 4, price: 9800, amenities: &["Wi-Fi", "TV", "AC"], status: "Reserved", discount_percent: 5, breakfast_included: true, uuid: "1a072438-7489-4f51-e280-b3ce4f56a778", created_at: "Oct 22, 2023 07:15 AM", updated_at: "May 8, 2024 11:40 AM" },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Guest {
    pub initials: &'static str,
    pub name: &'static str,
    pub nationality: &'static str,
    pub booking_ref: &'static str,
    pub room: &'static str,
    pub room_type: &'static str,
    pub stay_dates: &'static str,
    pub nights: &'static str,
    pub contact: &'static str,
    pub guests: &'static str,
    pub status: &'static str,
    pub vip: bool,
    pub id_passport: &'static str,
    pub date_of_birth: &'static str,
    pub preferred_language: &'static str,
    pub address: &'static str,
    pub member_since: &'static str,
    pub company: &'static str,
    pub notes: &'static str,
    pub check_in_date: &'static str,
    pub check_out_date: &'static str,
    pub payment_method: &'static str,
    pub special_notes: &'static str,
    pub total_stays: u32,
    pub total_nights: u32,
}

pub const GUESTS: &[Guest] = &[
    Guest { initials: "AH", name: "Ahmed Hassan", nationality: "Ethiopian", booking_ref: "HA-250143", room: "101", room_type: "Standard Room", stay_dates: "16 May – 18 May", nights: "2 nights", contact: "+251 91 234 5678", guests: "2 Guests", status: "In House", vip: true,
        id_passport: "EP1234567", date_of_birth: "12 Aug 1988", preferred_language: "English", address: "Bole Road, Addis Ababa, Ethiopia", member_since: "05 Mar 2023", company: "—",
        notes: "Prefers quiet rooms. Late check-out when available.", check_in_date: "16 May 2024, 2:00 PM", check_out_date: "18 May 2024, 12:00 PM", payment_method: "Visa •••• 4242 · Prepaid",
        special_notes: "Guest requested extra towels and mineral water on arrival.", total_stays: 4, total_nights: 11 },
    Guest { initials: "FA", name: "Fatima Ali", nationality: "Ethiopian", booking_ref: "HA-250144", room: "102", room_type: "Deluxe Room", stay_dates: "16 May – 17 May", nights: "1 night", contact: "+251 91 876 5432", guests: "1 Guest", status: "Arriving Today", vip: false,
        id_passport: "EP2345678", date_of_birth: "03 Feb 1995", preferred_language: "Amharic", address: "CMC Road, Addis Ababa, Ethiopia", member_since: "18 Jun 2023", company: "—",
        notes: "First-time guest, arriving by flight in the evening.", check_in_date: "16 May 2024, 3:00 PM", check_out_date: "17 May 2024, 12:00 PM", payment_method: "Mastercard •••• 7781 · Prepaid",
        special_notes: "Requested airport pickup.", total_stays: 1, total_nights: 1 },
    Guest { initials: "MN", name: "Mohamed Nur", nationality: "American", booking_ref: "HA-250145", room: "301", room_type: "Executive Suite", stay_dates: "17 May – 19 May", nights: "2 nights", contact: "+1 202 555 0134", guests: "2 Guests", status: "In House", vip: false,
        id_passport: "US9876543", date_of_birth: "27 Nov 1984", preferred_language: "English", address: "1600 K Street, Washington DC, USA", member_since: "11 Jan 2024", company: "Nur Consulting LLC",
        notes: "Celebrating wedding anniversary, requested room decoration.", check_in_date: "17 May 2024, 1:00 PM", check_out_date: "19 May 2024, 12:00 PM", payment_method: "Visa •••• 5521 · Prepaid",
        special_notes: "Anniversary — arranged sparkling wine on arrival.", total_stays: 2, total_nights: 5 },
    Guest { initials: "AH", name: "Abdirahman Hassan", nationality: "Ethiopian", booking_ref: "HA-250146", room: "104", room_type: "Family Room", stay_dates: "17 May – 18 May", nights: "1 night", contact: "+251 92 345 6789", guests: "4 Guests", status: "Checking Out", vip: false,
        id_passport: "EP3456789", date_of_birth: "09 May 1979", preferred_language: "Amharic", address: "Kazanchis, Addis Ababa, Ethiopia", member_since: "22 Sep 2022", company: "—",
        notes: "Travelling with family, requested a crib.", check_in_date: "17 May 2024, 2:00 PM", check_out_date: "18 May 2024, 12:00 PM", payment_method: "Cash on arrival",
        special_notes: "Late checkout requested, pending approval.", total_stays: 3, total_nights: 6 },
    Guest { initials: "OF", name: "Omar Farah", nationality: "British", booking_ref: "HA-250147", room: "201", room_type: "Twin Room", stay_dates: "18 May – 21 May", nights: "3 nights", contact: "+44 7700 900123", guests: "2 Guests", status: "In House", vip: false,
        id_passport: "GB1122334", date_of_birth: "15 Mar 1990", preferred_language: "English", address: "Baker Street, London, United Kingdom", member_since: "30 Oct 2023", company: "—",
        notes: "Mentioned the AC was noisy on a previous stay.", check_in_date: "18 May 2024, 2:30 PM", check_out_date: "21 May 2024, 12:00 PM", payment_method: "Visa •••• 9034 · Prepaid",
        special_notes: "Requested a quieter room away from the elevator.", total_stays: 2, total_nights: 5 },
    Guest { initials: "YM", name: "Yusuf Mohamed", nationality: "Canadian", booking_ref: "HA-250148", room: "205", room_type: "Single Room", stay_dates: "18 May – 20 May", nights: "2 nights", contact: "+1 416 555 0199", guests: "1 Guest", status: "VIP", vip: true,
        id_passport: "CA5566778", date_of_birth: "21 Jul 1982", preferred_language: "English", address: "Yonge Street, Toronto, Canada", member_since: "14 Feb 2022", company: "Mohamed & Partners",
        notes: "Frequent business traveller, prefers high floor rooms.", check_in_date: "18 May 2024, 12:00 PM", check_out_date: "20 May 2024, 12:00 PM", payment_method: "Amex •••• 1029 · Prepaid",
        special_notes: "Upgraded to a Deluxe Room as a VIP courtesy.", total_stays: 6, total_nights: 14 },
    Guest { initials: "HA", name: "Hawa Abdi", nationality: "Ethiopian", booking_ref: "HA-250149", room: "202", room_type: "Deluxe Room", stay_dates: "19 May – 21 May", nights: "2 nights", contact: "+251 93 456 7890", guests: "2 Guests", status: "Arriving Today", vip: false,
        id_passport: "EP4567890", date_of_birth: "02 Dec 1993", preferred_language: "Amharic", address: "Piazza, Addis Ababa, Ethiopia", member_since: "07 Jul 2023", company: "—",
        notes: "Booked via WhatsApp, confirmed by phone.", check_in_date: "19 May 2024, 2:00 PM", check_out_date: "21 May 2024, 12:00 PM", payment_method: "Telebirr · Prepaid",
        special_notes: "No special requests noted.", total_stays: 1, total_nights: 2 },
    Guest { initials: "SD", name: "Selam Desta", nationality: "Ethiopian", booking_ref: "HA-250150", room: "105", room_type: "Standard Room", stay_dates: "16 May – 16 May", nights: "0 night", contact: "+251 91 321 6543", guests: "1 Guest", status: "No-show", vip: false,
        id_passport: "EP5678901", date_of_birth: "30 Jan 1997", preferred_language: "Amharic", address: "Megenagna, Addis Ababa, Ethiopia", member_since: "19 Apr 2024", company: "—",
        notes: "Did not arrive for scheduled check-in.", check_in_date: "16 May 2024, 2:00 PM", check_out_date: "16 May 2024, 12:00 PM", payment_method: "Visa •••• 3312 · Prepaid",
        special_notes: "Marked as no-show, refund processed.", total_stays: 1, total_nights: 0 },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PastBooking {
    pub booking_ref: &'static str,
    pub room: &'static str,
    pub room_type: &'static str,
    pub stay_dates: &'static str,
    pub nights: u32,
    pub amount: u32,
    pub status: &'static str,
}

pub fn past_bookings(booking_ref: &str) -> &'static [PastBooking] {
    match booking_ref {
        "HA-250143" => &[
            PastBooking { booking_ref: "HA-240089", room: "305", room_type: "Executive Suite", stay_dates: "10 Apr – 12 Apr 2024", nights: 2, amount: 19_600, status: "Completed" },
            PastBooking { booking_ref: "HA-240047", room: "202", room_type: "Deluxe Room", stay_dates: "25 Feb – 26 Feb 2024", nights: 1, amount: 6_200, status: "Completed" },
            PastBooking { booking_ref: "HA-231112", room: "201", room_type: "Deluxe Room", stay_dates: "18 Dec – 21 Dec 2023", nights: 3, amount: 18_600, status: "Completed" },
            PastBooking { booking_ref: "HA-231045", room: "105", room_type: "Standard Room", stay_dates: "05 Sep – 06 Sep 2023", nights: 1, amount: 4_500, status: "Cancelled" },
            PastBooking { booking_ref: "HA-230912", room: "301", room_type: "Executive Suite", stay_dates: "12 Aug – 14 Aug 2023", nights: 2, amount: 19_600, status: "No-show" },
        ],
        "HA-250148" => &[
            PastBooking { booking_ref: "HA-241203", room: "203", room_type: "Deluxe Room", stay_dates: "20 Mar – 23 Mar 2024", nights: 3, amount: 18_600, status: "Completed" },
            PastBooking { booking_ref: "HA-232209", room: "205", room_type: "Single Room", stay_dates: "02 Nov – 04 Nov 2023", nights: 2, amount: 7_800, status: "Completed" },
        ],
        "HA-250147" => &[
            PastBooking { booking_ref: "HA-240711", room: "201", room_type: "Twin Room", stay_dates: "14 Jan – 16 Jan 2024", nights: 2, amount: 10_200, status: "Completed" },
        ],
        _ => &[],
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reservation {
    pub booking_ref: &'static str,
    pub guest: &'static str,
    pub check_in: &'static str,
    pub check_out: &'static str,
    pub guests: u32,
    pub status: &'static str,
    pub time: &'static str,
    pub phone: &'static str,
}

pub const RESERVATIONS: &[Reservation] = &[
    Reservation { booking_ref: "HA-250143", guest: "Ahmed Hassan", check_in: "16 May", check_out: "18 May", guests: 2, status: "Confirmed", time: "10:30 AM", phone: "0912345678" },
    Reservation { booking_ref: "HA-250144", guest: "Fatima Ali", check_in: "16 May", check_out: "17 May", guests: 1, status: "New", time: "09:20 AM", phone: "0918765432" },
    Reservation { booking_ref: "HA-250145", guest: "Mohamed Nur", check_in: "17 May", check_out: "19 May", guests: 2, status: "Confirmed", time: "08:45 AM", phone: "0911122333" },
    Reservation { booking_ref: "HA-250146", guest: "Abdirahman Hassan", check_in: "17 May", check_out: "18 May", guests: 1, status: "Confirmed", time: "08:15 AM", phone: "0915566778" },
    Reservation { booking_ref: "HA-250147", guest: "Yusuf Mohamed", check_in: "18 May", check_out: "20 May", guests: 2, status: "New", time: "11:05 AM", phone: "0914455667" },
    Reservation { booking_ref: "HA-250148", guest: "Hawa Abdi", check_in: "18 May", check_out: "19 May", guests: 1, status: "Cancelled", time: "12:00 PM", phone: "0919988776" },
    Reservation { booking_ref: "HA-250149", guest: "Omar Farah", check_in: "19 May", check_out: "21 May", guests: 2, status: "Confirmed", time: "01:15 PM", phone: "0912233445" },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Admin {
    pub initials: &'static str,
    pub name: &'static str,
    pub role: &'static str,
    pub email: &'static str,
    pub phone: &'static str,
    pub last_active: &'static str,
    pub status: &'static str,
}

pub const ADMINS: &[Admin] = &[
    Admin { initials: "AH", name: "Ahmed Hassan", role: "System Administrator", email: "ahmed.hassan@tourista.com", phone: "+251 91 234 5678", last_active: "Today, 10:42 AM", status: "Active" },
    Admin { initials: "MA", name: "Mekdes Alemu", role: "Hotel Manager", email: "mekdes.alemu@tourista.com", phone: "+251 91 987 6543", last_active: "Today, 08:15 AM", status: "Active" },
    Admin { initials: "BT", name: "Bereket Tesfaye", role: "Reservation Manager", email: "bereket.tesfaye@tourista.com", phone: "+251 92 345 6789", last_active: "Yesterday, 07:30 PM", status: "Active" },
    Admin { initials: "YK", name: "Yordanos Kebede", role: "Finance Admin", email: "yordanos.kebede@tourista.com", phone: "+251 91 456 7890", last_active: "Yesterday, 03:20 PM", status: "Active" },
    Admin { initials: "SA", name: "Samuel Assefa", role: "Support Staff", email: "samuel.assefa@tourista.com", phone: "+251 91 112 3344", last_active: "2 days ago, 11:05 AM", status: "Active" },
    Admin { initials: "HB", name: "Helen Bekele", role: "Content Manager", email: "helen.bekele@tourista.com", phone: "+251 91 778 8990", last_active: "3 days ago, 09:50 AM", status: "Inactive" },
];

pub fn find_room(number: &str) -> Option<&'static Room> {
    ROOMS.iter().find(|r| r.number == number)
}

pub fn find_guest(booking_ref: &str) -> Option<&'static Guest> {
    GUESTS.iter().find(|g| g.booking_ref == booking_ref)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Review {
    pub guest: &'static str,
    pub initials: &'static str,
    pub rating: u32,
    pub date: &'static str,
    pub room_type: &'static str,
    pub comment: &'static str,
    pub reply: Option<&'static str>,
}

pub const REVIEWS: &[Review] = &[
    Review { guest: "Ahmed Hassan", initials: "AH", rating: 5, date: "18 May 2025", room_type: "Standard Room", comment: "Excellent stay! The staff were incredibly welcoming and the room was spotless. Will definitely come back.", reply: Some("Thank you so much, Ahmed! We look forward to welcoming you again.") },
    Review { guest: "Fatima Ali", initials: "FA", rating: 4, date: "17 May 2025", room_type: "Deluxe Room", comment: "Great location and comfortable bed. Breakfast could have more variety.", reply: None },
    Review { guest: "Mohamed Nur", initials: "MN", rating: 5, date: "19 May 2025", room_type: "Executive Suite", comment: "The Executive Suite was spacious and the view was stunning. Perfect for our anniversary trip.", reply: Some("We're thrilled you enjoyed your anniversary with us, Mohamed!") },
    Review { guest: "Omar Farah", initials: "OF", rating: 3, date: "21 May 2025", room_type: "Twin Room", comment: "Room was fine but the AC was a bit noisy at night. Front desk was helpful when I mentioned it.", reply: None },
    Review { guest: "Hawa Abdi", initials: "HA", rating: 5, date: "21 May 2025", room_type: "Deluxe Room", comment: "Loved every minute of it. The WhatsApp booking confirmation made everything so easy.", reply: None },
    Review { guest: "Selam Desta", initials: "SD", rating: 2, date: "16 May 2025", room_type: "Standard Room", comment: "Had to cancel due to a scheduling conflict, but the refund process was quick at least.", reply: Some("Sorry we missed you, Selam — hope to host you another time.") },
];

#[derive(Debug, Clone, PartialEq)]
pub struct ChatMessage {
    pub from_guest: bool,
    pub text: &'static str,
    pub time: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conversation {
    pub guest: &'static str,
    pub initials: &'static str,
    pub booking_ref: &'static str,
    pub last_message: &'static str,
    pub time: &'static str,
    pub unread: u32,
    pub thread: &'static [ChatMessage],
}

pub const CONVERSATIONS: &[Conversation] = &[
    Conversation {
        guest: "Ahmed Hassan", initials: "AH", booking_ref: "HA-250143",
        last_message: "Could I get a late checkout tomorrow?", time: "10:42 AM", unread: 2,
        thread: &[
            ChatMessage { from_guest: true, text: "Hi, I just checked in — room 101 is lovely, thank you!", time: "Yesterday, 3:12 PM" },
            ChatMessage { from_guest: false, text: "Welcome, Mr. Hassan! Glad you like the room. Let us know if you need anything.", time: "Yesterday, 3:20 PM" },
            ChatMessage { from_guest: true, text: "Could I get a late checkout tomorrow?", time: "10:42 AM" },
        ],
    },
    Conversation {
        guest: "Fatima Ali", initials: "FA", booking_ref: "HA-250144",
        last_message: "Thank you, see you soon!", time: "09:25 AM", unread: 0,
        thread: &[
            ChatMessage { from_guest: true, text: "Is airport pickup available for my arrival tonight?", time: "09:10 AM" },
            ChatMessage { from_guest: false, text: "Yes! Please share your flight number and we'll arrange pickup.", time: "09:18 AM" },
            ChatMessage { from_guest: true, text: "Thank you, see you soon!", time: "09:25 AM" },
        ],
    },
    Conversation {
        guest: "Yusuf Mohamed", initials: "YM", booking_ref: "HA-250148",
        last_message: "Perfect, thank you for the upgrade!", time: "Yesterday", unread: 0,
        thread: &[
            ChatMessage { from_guest: false, text: "Hi Yusuf, as a VIP guest we've upgraded you to a Deluxe Room at no extra cost.", time: "Yesterday, 1:00 PM" },
            ChatMessage { from_guest: true, text: "Perfect, thank you for the upgrade!", time: "Yesterday, 1:05 PM" },
        ],
    },
];

// ---------------------------------------------------------------------------
// Analytics
// ---------------------------------------------------------------------------

/// Revenue in thousands of ETB, most recent last.
pub const REVENUE_7D: &[u32] = &[38, 42, 36, 51, 47, 62, 58];
pub const REVENUE_30D: &[u32] = &[
    31, 34, 29, 38, 42, 47, 44, 39, 36, 41, 45, 52, 49, 43, 40, 44, 48, 55, 51, 46, 42, 47, 53, 58,
    54, 49, 45, 51, 57, 62,
];
pub const REVENUE_12M: &[u32] = &[820, 760, 910, 1040, 980, 1120, 1310, 1250, 1180, 1420, 1380, 1510];

pub const OCCUPANCY_7D: &[u32] = &[62, 68, 59, 74, 71, 88, 85];
pub const OCCUPANCY_30D: &[u32] = &[
    55, 58, 52, 61, 66, 70, 68, 63, 60, 64, 69, 75, 72, 67, 64, 68, 71, 78, 74, 70, 66, 71, 76, 82,
    79, 73, 69, 75, 81, 85,
];
pub const OCCUPANCY_12M: &[u32] = &[54, 51, 58, 63, 60, 66, 74, 71, 68, 78, 76, 82];

pub const BOOKINGS_7D: &[u32] = &[9, 12, 8, 14, 11, 19, 16];

pub const DAY_LABELS: &[&str] = &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
pub const MONTH_LABELS: &[&str] = &[
    "Sep", "Oct", "Nov", "Dec", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug",
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelSplit {
    pub name: &'static str,
    pub bookings: u32,
    pub color: &'static str,
}

pub const CHANNELS: &[ChannelSplit] = &[
    ChannelSplit { name: "Portal (direct)", bookings: 148, color: "#2563eb" },
    ChannelSplit { name: "Walk-in", bookings: 62, color: "#0ea5e9" },
    ChannelSplit { name: "Phone & WhatsApp", bookings: 47, color: "#8b5cf6" },
    ChannelSplit { name: "Corporate accounts", bookings: 29, color: "#f59e0b" },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoomTypePerf {
    pub name: &'static str,
    pub sold: u32,
    pub revenue: u32,
    pub occupancy: u32,
    pub adr: u32,
}

pub const ROOM_TYPE_PERFORMANCE: &[RoomTypePerf] = &[
    RoomTypePerf { name: "Deluxe Room", sold: 96, revenue: 595_200, occupancy: 88, adr: 6_200 },
    RoomTypePerf { name: "Standard Room", sold: 121, revenue: 544_500, occupancy: 82, adr: 4_500 },
    RoomTypePerf { name: "Executive Suite", sold: 38, revenue: 372_400, occupancy: 64, adr: 9_800 },
    RoomTypePerf { name: "Family Room", sold: 44, revenue: 325_600, occupancy: 71, adr: 7_400 },
    RoomTypePerf { name: "Twin Room", sold: 57, revenue: 290_700, occupancy: 76, adr: 5_100 },
    RoomTypePerf { name: "Single Room", sold: 63, revenue: 245_700, occupancy: 69, adr: 3_900 },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NationalitySplit {
    pub country: &'static str,
    pub guests: u32,
}

pub const NATIONALITIES: &[NationalitySplit] = &[
    NationalitySplit { country: "Ethiopia", guests: 184 },
    NationalitySplit { country: "Kenya", guests: 46 },
    NationalitySplit { country: "United States", guests: 33 },
    NationalitySplit { country: "United Kingdom", guests: 28 },
    NationalitySplit { country: "Somalia", guests: 24 },
    NationalitySplit { country: "Canada", guests: 19 },
    NationalitySplit { country: "Djibouti", guests: 12 },
];

// ---------------------------------------------------------------------------
// Activity feed
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Activity {
    pub icon: &'static str,
    pub tint: &'static str,
    pub title: &'static str,
    pub detail: &'static str,
    pub time: &'static str,
}

pub const ACTIVITY: &[Activity] = &[
    Activity { icon: "calendar-check", tint: "bg-blue-50 text-blue-600", title: "New reservation", detail: "Fatima Ali · Deluxe Room 102 · 16–17 May", time: "4 min ago" },
    Activity { icon: "check-circle", tint: "bg-emerald-50 text-emerald-600", title: "Guest checked in", detail: "Ahmed Hassan · Room 101", time: "22 min ago" },
    Activity { icon: "star", tint: "bg-amber-50 text-amber-600", title: "5-star review received", detail: "Mohamed Nur · Executive Suite", time: "1 hour ago" },
    Activity { icon: "wrench", tint: "bg-red-50 text-red-600", title: "Room flagged for maintenance", detail: "Room 301 · AC unit reported noisy", time: "2 hours ago" },
    Activity { icon: "mail", tint: "bg-purple-50 text-purple-600", title: "Guest message", detail: "Ahmed Hassan asked about late checkout", time: "3 hours ago" },
    Activity { icon: "x-circle", tint: "bg-slate-100 text-slate-500", title: "Reservation cancelled", detail: "Hawa Abdi · HA-250148 · free cancellation", time: "Yesterday" },
    Activity { icon: "wallet", tint: "bg-emerald-50 text-emerald-600", title: "Payout processed", detail: "ETB 184,300 for week ending 10 May", time: "2 days ago" },
];

// ---------------------------------------------------------------------------
// Payments
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Payment {
    pub invoice: &'static str,
    pub booking_ref: &'static str,
    pub guest: &'static str,
    pub initials: &'static str,
    pub room: &'static str,
    pub date: &'static str,
    pub method: &'static str,
    pub amount: u32,
    pub status: &'static str,
}

pub const PAYMENTS: &[Payment] = &[
    Payment { invoice: "INV-2026-0431", booking_ref: "HA-250143", guest: "Ahmed Hassan", initials: "AH", room: "101", date: "18 May 2026", method: "Visa •••• 4242", amount: 10_350, status: "Paid" },
    Payment { invoice: "INV-2026-0430", booking_ref: "HA-250144", guest: "Fatima Ali", initials: "FA", room: "102", date: "17 May 2026", method: "Mastercard •••• 7781", amount: 7_130, status: "Paid" },
    Payment { invoice: "INV-2026-0429", booking_ref: "HA-250145", guest: "Mohamed Nur", initials: "MN", room: "301", date: "19 May 2026", method: "Cash", amount: 22_540, status: "Pending" },
    Payment { invoice: "INV-2026-0428", booking_ref: "HA-250146", guest: "Abdirahman Hassan", initials: "AH", room: "104", date: "18 May 2026", method: "Telebirr", amount: 8_510, status: "Paid" },
    Payment { invoice: "INV-2026-0427", booking_ref: "HA-250147", guest: "Omar Farah", initials: "OF", room: "201", date: "21 May 2026", method: "Visa •••• 9034", amount: 17_595, status: "Pending" },
    Payment { invoice: "INV-2026-0426", booking_ref: "HA-250148", guest: "Yusuf Mohamed", initials: "YM", room: "205", date: "20 May 2026", method: "Amex •••• 1029", amount: 8_970, status: "Refunded" },
    Payment { invoice: "INV-2026-0425", booking_ref: "HA-250149", guest: "Hawa Abdi", initials: "HA", room: "202", date: "21 May 2026", method: "Telebirr", amount: 14_260, status: "Paid" },
    Payment { invoice: "INV-2026-0424", booking_ref: "HA-250150", guest: "Selam Desta", initials: "SD", room: "105", date: "16 May 2026", method: "Visa •••• 3312", amount: 5_175, status: "Failed" },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Payout {
    pub period: &'static str,
    pub reference: &'static str,
    pub bookings: u32,
    pub gross: u32,
    pub status: &'static str,
    pub settled_on: &'static str,
}

pub const PAYOUTS: &[Payout] = &[
    Payout { period: "11 – 17 May 2026", reference: "PO-2026-19", bookings: 47, gross: 214_800, status: "Scheduled", settled_on: "24 May 2026" },
    Payout { period: "4 – 10 May 2026", reference: "PO-2026-18", bookings: 41, gross: 184_300, status: "Settled", settled_on: "17 May 2026" },
    Payout { period: "27 Apr – 3 May 2026", reference: "PO-2026-17", bookings: 38, gross: 171_500, status: "Settled", settled_on: "10 May 2026" },
    Payout { period: "20 – 26 Apr 2026", reference: "PO-2026-16", bookings: 44, gross: 196_200, status: "Settled", settled_on: "3 May 2026" },
];

// ---------------------------------------------------------------------------
// Rates & availability
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RatePlan {
    pub name: &'static str,
    pub code: &'static str,
    pub description: &'static str,
    pub adjustment: i32,
    pub min_stay: u32,
    pub refundable: bool,
    pub active: bool,
    pub bookings: u32,
}

pub const RATE_PLANS: &[RatePlan] = &[
    RatePlan { name: "Standard Flexible", code: "FLEX", description: "Free cancellation up to 24 hours before arrival.", adjustment: 0, min_stay: 1, refundable: true, active: true, bookings: 164 },
    RatePlan { name: "Advance Purchase", code: "ADV14", description: "Book at least 14 days ahead. Non-refundable.", adjustment: -15, min_stay: 2, refundable: false, active: true, bookings: 78 },
    RatePlan { name: "Weekend Getaway", code: "WKND", description: "Friday and Saturday nights, breakfast for two included.", adjustment: -8, min_stay: 2, refundable: true, active: true, bookings: 52 },
    RatePlan { name: "Corporate Negotiated", code: "CORP", description: "Contracted rate for registered corporate accounts.", adjustment: -22, min_stay: 1, refundable: true, active: true, bookings: 41 },
    RatePlan { name: "Peak Season", code: "PEAK", description: "Applies over conference weeks and public holidays.", adjustment: 25, min_stay: 3, refundable: false, active: false, bookings: 0 },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AvailabilityDay {
    pub label: &'static str,
    pub date: &'static str,
    pub total: u32,
    pub sold: u32,
    pub rate: u32,
    pub closed: bool,
}

pub const AVAILABILITY: &[AvailabilityDay] = &[
    AvailabilityDay { label: "Mon", date: "1 Sep", total: 48, sold: 31, rate: 4500, closed: false },
    AvailabilityDay { label: "Tue", date: "2 Sep", total: 48, sold: 35, rate: 4500, closed: false },
    AvailabilityDay { label: "Wed", date: "3 Sep", total: 48, sold: 29, rate: 4500, closed: false },
    AvailabilityDay { label: "Thu", date: "4 Sep", total: 48, sold: 41, rate: 4900, closed: false },
    AvailabilityDay { label: "Fri", date: "5 Sep", total: 48, sold: 45, rate: 5400, closed: false },
    AvailabilityDay { label: "Sat", date: "6 Sep", total: 48, sold: 48, rate: 5400, closed: true },
    AvailabilityDay { label: "Sun", date: "7 Sep", total: 48, sold: 38, rate: 4900, closed: false },
    AvailabilityDay { label: "Mon", date: "8 Sep", total: 48, sold: 26, rate: 4500, closed: false },
    AvailabilityDay { label: "Tue", date: "9 Sep", total: 48, sold: 30, rate: 4500, closed: false },
    AvailabilityDay { label: "Wed", date: "10 Sep", total: 48, sold: 33, rate: 4500, closed: false },
    AvailabilityDay { label: "Thu", date: "11 Sep", total: 48, sold: 39, rate: 4900, closed: false },
    AvailabilityDay { label: "Fri", date: "12 Sep", total: 48, sold: 44, rate: 5400, closed: false },
    AvailabilityDay { label: "Sat", date: "13 Sep", total: 48, sold: 46, rate: 5400, closed: false },
    AvailabilityDay { label: "Sun", date: "14 Sep", total: 48, sold: 35, rate: 4900, closed: false },
];

// ---------------------------------------------------------------------------
// Staff
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Role {
    pub name: &'static str,
    pub description: &'static str,
    pub permissions: &'static [&'static str],
    pub members: u32,
}

pub const ROLES: &[Role] = &[
    Role {
        name: "System Administrator",
        description: "Full access to every area including billing and staff management.",
        permissions: &["Reservations", "Rooms & rates", "Guests", "Payments", "Reviews", "Staff", "Settings"],
        members: 1,
    },
    Role {
        name: "Hotel Manager",
        description: "Day-to-day operations and reporting, without staff or billing controls.",
        permissions: &["Reservations", "Rooms & rates", "Guests", "Reviews", "Analytics"],
        members: 1,
    },
    Role {
        name: "Reservation Manager",
        description: "Handles bookings, the calendar and guest correspondence.",
        permissions: &["Reservations", "Calendar", "Guests", "Messages"],
        members: 1,
    },
    Role {
        name: "Finance Admin",
        description: "Payments, invoices and payouts only.",
        permissions: &["Payments", "Analytics"],
        members: 1,
    },
    Role {
        name: "Support Staff",
        description: "Read-only access plus guest messaging.",
        permissions: &["Reservations (read)", "Guests (read)", "Messages"],
        members: 1,
    },
    Role {
        name: "Content Manager",
        description: "Maintains the public listing, photographs and descriptions.",
        permissions: &["Hotel profile", "Rooms (content)", "Reviews (reply)"],
        members: 1,
    },
];

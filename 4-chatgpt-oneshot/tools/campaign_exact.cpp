#include <algorithm>
#include <array>
#include <bit>
#include <chrono>
#include <cstdint>
#include <iostream>
#include <numeric>
#include <sstream>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <vector>

using Cards = std::array<uint8_t, 24>;
using Clock = std::chrono::steady_clock;

struct State {
    Cards cards{};
    int a = 0;
    int d = 0;
};

struct Pattern {
    uint32_t mask;
    Cards labels{};
};

struct StateKey {
    uint64_t low;
    uint64_t high;
    bool operator==(const StateKey&) const = default;
};

struct StateHash {
    size_t operator()(const StateKey& key) const {
        uint64_t value = key.low ^ (key.high + 0x9e3779b97f4a7c15ULL);
        value ^= value >> 30;
        value *= 0xbf58476d1ce4e5b9ULL;
        value ^= value >> 27;
        value *= 0x94d049bb133111ebULL;
        return value ^ (value >> 31);
    }
};

struct Database {
    int n;
    std::array<uint64_t, 13> factorial{};
    std::array<std::array<int, 13>, 13> cut_id{};
    std::vector<std::pair<int, int>> cuts;
    std::vector<uint8_t> distances;

    explicit Database(int size) : n(size) {
        factorial.at(0) = 1;
        for (int i = 1; i <= 12; ++i)
            factorial.at(i) = factorial.at(i - 1) * i;
        for (int a = 0; a <= n; ++a)
            for (int d = 0; d <= n - a; ++d) {
                cut_id.at(a).at(d) = cuts.size();
                cuts.emplace_back(a, d);
            }
    }

    uint64_t rank(const Cards& p) const {
        uint32_t remaining = (1u << n) - 1;
        uint64_t value = 0;
        for (int i = 0; i < n; ++i) {
            int card = p.at(i);
            value += std::popcount(remaining & ((1u << card) - 1))
                     * factorial.at(n - i - 1);
            remaining &= ~(1u << card);
        }
        return value;
    }

    uint64_t id(const State& state) const {
        return rank(state.cards) * cuts.size()
               + cut_id.at(state.a).at(state.d);
    }

    State state(uint64_t id) const {
        State result;
        auto cut = cuts.at(id % cuts.size());
        result.a = cut.first;
        result.d = cut.second;
        uint64_t value = id / cuts.size();
        uint32_t remaining = (1u << n) - 1;
        for (int i = 0; i < n; ++i) {
            uint64_t selection = value / factorial.at(n - i - 1);
            value %= factorial.at(n - i - 1);
            uint32_t choices = remaining;
            while (selection--) choices &= choices - 1;
            result.cards.at(i) = std::countr_zero(choices);
            remaining &= ~(1u << result.cards.at(i));
        }
        return result;
    }
};

void rotate(Cards& cards, int first, int middle, int last) {
    auto original = cards;
    int length = last - first;
    for (int i = 0; i < length; ++i)
        cards.at(first + i) = original.at(first + (i + middle - first) % length);
}

bool move(State& state, int code, int n) {
    int a = state.a;
    int d = state.d;
    if (code == 0 && a) {
        rotate(state.cards, 0, 1, a);
        --state.a;
        ++state.d;
    } else if (code == 1 && d) {
        rotate(state.cards, 0, a, a + 1);
        ++state.a;
        --state.d;
    } else if (code == 2 && d) {
        rotate(state.cards, a, a + 1, a + d);
        --state.d;
    } else if (code == 3 && n - a - d) {
        rotate(state.cards, a, a + d, a + d + 1);
        ++state.d;
    } else return false;
    return true;
}

void build(Database& db) {
    uint64_t count = db.factorial.at(db.n) * db.cuts.size();
    db.distances.assign(count, 255);
    std::vector<uint32_t> queue;
    queue.reserve(count);
    State goal;
    goal.d = db.n;
    std::iota(goal.cards.begin(), goal.cards.begin() + db.n, 0);
    uint32_t initial = db.id(goal);
    db.distances.at(initial) = 0;
    queue.push_back(initial);
    for (size_t head = 0; head < queue.size(); ++head) {
        uint32_t current = queue.at(head);
        State state = db.state(current);
        for (int code = 0; code < 4; ++code) {
            auto successor = state;
            if (!move(successor, code, db.n)) continue;
            uint32_t next = db.id(successor);
            if (db.distances.at(next) == 255) {
                db.distances.at(next) = db.distances.at(current) + 1;
                queue.push_back(next);
            }
        }
    }
    if (queue.size() != count) throw std::runtime_error("incomplete database");
}

std::vector<std::pair<int, int>> side_options(const State& state,
                                            int first, int last) {
    std::vector<std::pair<int, int>> result{{-1, 0}};
    for (int index = first; index < last; ++index) {
        int threshold = state.cards.at(index);
        std::array<int, 24> lengths{};
        int maximum = 0;
        for (int position = first; position < last; ++position) {
            int card = state.cards.at(position);
            if (card > threshold) continue;
            lengths.at(position) = 1;
            for (int earlier = first; earlier < position; ++earlier)
                if (state.cards.at(earlier) > card)
                    lengths.at(position) = std::max(lengths.at(position),
                                                    lengths.at(earlier) + 1);
            maximum = std::max(maximum, lengths.at(position));
        }
        result.emplace_back(threshold, maximum);
    }
    return result;
}

int residual_bound(const State& state, int n) {
    int suffix = 0;
    while (suffix < state.d
           && state.cards.at(state.a + state.d - 1 - suffix) == n - 1 - suffix)
        ++suffix;
    int active_d = state.d - suffix;
    int side_cards = n - state.d;
    int baseline = side_cards + 2 * active_d;
    auto left_options = side_options(state, 0, state.a);
    auto right_options = side_options(state, state.a + state.d, n);
    int width = n + 1;
    std::array<int8_t, 625> current;
    current.fill(-1);
    for (auto [left, left_count] : left_options)
        for (auto [right, right_count] : right_options)
            current.at((left + 1) * width + right + 1) = left_count + right_count;
    for (int index = state.a; index < state.a + active_d; ++index) {
        int card = state.cards.at(index) + 1;
        auto following = current;
        for (int first = 0; first < width; ++first)
            for (int second = 0; second < width; ++second) {
                int count = current.at(first * width + second);
                if (count < 0) continue;
                if (card > first)
                    following.at(card * width + second) = std::max<int>(
                        following.at(card * width + second), count + 1);
                if (card > second)
                    following.at(first * width + card) = std::max<int>(
                        following.at(first * width + card), count + 1);
            }
        current = following;
    }
    int maximum = *std::max_element(current.begin(), current.end());
    return baseline + 2 * (side_cards + active_d - maximum);
}

struct Search {
    int n;
    Database& pdb;
    std::vector<Pattern> patterns;
    Clock::time_point deadline;
    std::unordered_map<StateKey, uint16_t, StateHash> seen;
    std::vector<int> path;
    uint64_t nodes = 0;
    uint64_t transposition_hits = 0;
    bool interrupted = false;
    int limit = 0;
    size_t capacity = 1000000;
    bool use_residual_structural = false;

    Search(int size, Database& database) : n(size), pdb(database) {}

    StateKey key(const State& state, int previous) const {
        StateKey result{0, 0};
        for (int i = 0; i < std::min(12, n); ++i)
            result.low = (result.low << 5) | state.cards.at(i);
        for (int i = 12; i < n; ++i)
            result.high = (result.high << 5) | state.cards.at(i);
        result.high = (result.high << 5) | state.a;
        result.high = (result.high << 5) | state.d;
        result.high = (result.high << 3) | (previous + 1);
        return result;
    }

    int pattern_value(const State& state, const Pattern& pattern,
                      const Cards& minimum, int baseline) const {
        State abstract;
        int length = 0;
        int outside = baseline;
        for (int i = 0; i < n; ++i) {
            int card = state.cards.at(i);
            if (!(pattern.mask & (1u << card))) continue;
            abstract.cards.at(length++) = pattern.labels.at(card);
            outside -= minimum.at(card);
            if (i < state.a) ++abstract.a;
            else if (i < state.a + state.d) ++abstract.d;
        }
        return pdb.distances.at(pdb.id(abstract)) + outside;
    }

    int heuristic(const State& state) const {
        Cards minimum{};
        int suffix = 0;
        while (suffix < state.d
               && state.cards.at(state.a + state.d - 1 - suffix) == n - 1 - suffix)
            ++suffix;
        int baseline = 0;
        for (int i = 0; i < n; ++i) {
            int cost = i < state.a || i >= state.a + state.d ? 1
                       : i < state.a + state.d - suffix ? 2 : 0;
            minimum.at(state.cards.at(i)) = cost;
            baseline += cost;
        }
        int result = baseline;
        for (const auto& pattern : patterns)
            result = std::max(result, pattern_value(state, pattern, minimum, baseline));
        if (use_residual_structural)
            result = std::max(result, residual_bound(state, n));
        int parity = (n - state.d) % 2;
        if (result % 2 != parity) ++result;
        return result;
    }

    bool dfs(const State& state, int depth, int previous) {
        ++nodes;
        if ((nodes & 4095) == 0 && Clock::now() >= deadline) {
            interrupted = true;
            return false;
        }
        int estimate = heuristic(state);
        if (depth + estimate > limit) return false;
        if (estimate == 0) return true;
        auto state_key = key(state, previous);
        auto found = seen.find(state_key);
        if (found != seen.end() && found->second <= depth) {
            ++transposition_hits;
            return false;
        }
        if (found != seen.end()) found->second = depth;
        else if (seen.size() < capacity) seen.emplace(state_key, depth);
        std::array<std::pair<int, int>, 4> choices{};
        int count = 0;
        for (int code = 0; code < 4; ++code) {
            if (previous >= 0 && code == (previous ^ 1)) continue;
            auto successor = state;
            if (move(successor, code, n))
                choices.at(count++) = {heuristic(successor), code};
        }
        for (int i = 1; i < count; ++i)
            for (int j = i; j > 0 && choices.at(j) < choices.at(j - 1); --j)
                std::swap(choices.at(j), choices.at(j - 1));
        for (int i = 0; i < count; ++i) {
            int code = choices.at(i).second;
            auto successor = state;
            move(successor, code, n);
            path.push_back(code);
            if (dfs(successor, depth + 1, code)) return true;
            path.pop_back();
            if (interrupted) return false;
        }
        return false;
    }
};

int main(int argc, char** argv) {
    try {
        if (argc != 7 && argc != 8)
            throw std::runtime_error("usage: campaign_exact TARGET_CSV UPPER LOWER SECONDS PATTERNS TT_CAPACITY [RESIDUAL_STRUCTURAL]");
        std::stringstream input(argv[1]);
        std::vector<int> target;
        std::string item;
        while (std::getline(input, item, ',')) target.push_back(std::stoi(item));
        int n = target.size();
        if (n < 1 || n > 20) throw std::runtime_error("unsupported size");
        auto sorted = target;
        std::sort(sorted.begin(), sorted.end());
        for (int i = 0; i < n; ++i)
            if (sorted.at(i) != i) throw std::runtime_error("invalid target");
        int upper = std::stoi(argv[2]);
        int lower = std::stoi(argv[3]);
        double seconds = std::stod(argv[4]);
        int pattern_limit = std::stoi(argv[5]);
        auto started = Clock::now();
        Database pdb(std::min(8, n));
        build(pdb);
        auto built = Clock::now();
        Search search(n, pdb);
        search.capacity = std::stoull(argv[6]);
        State initial;
        initial.d = n;
        for (int i = 0; i < n; ++i) initial.cards.at(target.at(i)) = i;
        std::vector<std::pair<int, Pattern>> candidates;
        for (uint32_t mask = 0; mask < (1u << n); ++mask) {
            if (std::popcount(mask) != pdb.n) continue;
            Pattern pattern{mask, {}};
            int label = 0;
            for (int card = 0; card < n; ++card)
                if (mask & (1u << card)) pattern.labels.at(card) = label++;
            search.patterns = {pattern};
            candidates.push_back({search.heuristic(initial), pattern});
        }
        std::stable_sort(candidates.begin(), candidates.end(),
                         [](const auto& a, const auto& b) { return a.first > b.first; });
        search.patterns.clear();
        for (const auto& candidate : candidates) {
            if (int(search.patterns.size()) >= pattern_limit) break;
            search.patterns.push_back(candidate.second);
        }
        if (argc == 8) {
            int enabled = std::stoi(argv[7]);
            if (enabled != 0 && enabled != 1)
                throw std::runtime_error("RESIDUAL_STRUCTURAL must be 0 or 1");
            search.use_residual_structural = enabled != 0;
        }
        int initial_heuristic = search.heuristic(initial);
        lower = std::max(lower, initial_heuristic);
        if (lower % 2) ++lower;
        if (lower > upper) throw std::runtime_error("lower exceeds upper");
        search.deadline = Clock::now() + std::chrono::milliseconds(int(seconds * 1000));
        std::vector<int> exhausted;
        bool solved = false;
        for (int threshold = lower; threshold < upper; threshold += 2) {
            search.limit = threshold;
            search.seen.clear();
            search.path.clear();
            if (search.dfs(initial, 0, -1)) {
                upper = search.path.size();
                lower = upper;
                solved = true;
                break;
            }
            if (search.interrupted) break;
            exhausted.push_back(threshold);
            lower = threshold + 2;
        }
        std::cout << "{\"n\":" << n << ",\"lower\":" << lower
                  << ",\"upper\":" << upper << ",\"initial_heuristic\":" << initial_heuristic
                  << ",\"pdb_states\":" << pdb.distances.size()
                  << ",\"patterns\":" << search.patterns.size()
                  << ",\"residual_structural\":" << (search.use_residual_structural ? "true" : "false")
                  << ",\"nodes\":" << search.nodes
                  << ",\"transposition_hits\":" << search.transposition_hits
                  << ",\"tt_entries\":" << search.seen.size()
                  << ",\"interrupted\":" << (search.interrupted ? "true" : "false")
                  << ",\"search_found_plan\":" << (solved ? "true" : "false")
                  << ",\"pdb_seconds\":" << std::chrono::duration<double>(built - started).count()
                  << ",\"total_seconds\":" << std::chrono::duration<double>(Clock::now() - started).count()
                  << ",\"exhausted_thresholds\":[";
        for (size_t i = 0; i < exhausted.size(); ++i)
            std::cout << (i ? "," : "") << exhausted.at(i);
        std::cout << "],\"word\":[";
        const std::array<std::string, 4> names{"AD", "DA", "DB", "BD"};
        if (solved)
            for (size_t i = 0; i < search.path.size(); ++i)
                std::cout << (i ? "," : "") << '"' << names.at(search.path.at(i)) << '"';
        std::cout << "]}\n";
        return 0;
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}
